# Handoff

Keep this short and current. Overwrite (don't append history) each time you hand off —
git history is the log if you need the past. See `agent/WORKFLOW.md` for when to update this.

## As of 2026-11-19 (Phase 7 ACTIVE, task 0012, slice 4 done)

**Branch:** `rewrite/rust`. Slice 4 at `79fe7ad`, CI run 44620654551 green on 3 OS + licences
(incl. the new cargo-about notices step).

**Last session:** slice 4 — `.github/workflows/release.yml` (tag `v*` → macOS arm64 dmg,
Windows NSIS, Linux AppImage + tar.gz → draft GitHub Release with `SHA256SUMS`; signing
hooks behind secrets), `cargo-about` notices shipped in every package (`about.toml`,
`apps/desktop/packaging/about.hbs`, CI `deny` job checks them), Windows `.ico`, CI also on
`main`. macOS packaging verified locally (`cargo packager --release`; `cargo-about` 0.9.3
installed with `--features cli`). Findings in the ADR-0021 slice-4 amendment. Then the
new icon design (user): redrawn by `render-icon.py`, all PNGs + `.ico` regenerated.

**Next session = CLEANUP** (user decision 2026-11-19; one session, may need several
commits). Read task 0012 "User decisions (2026-11-19, after slice 3)" first. Order:

1. **Close Phase 7.** Task 0012 → `tasks/done/` with slices 5–7 deferred (no Win/Linux
   devices, no external webcam) and 8 dropped (user), reasons explicit; acceptance criteria
   marked met / deferred; `docs/ROADMAP.md` Phase 7 status. Release-criteria text the gate
   asks for: a short known-exceptions section (Gatekeeper on ad-hoc, SmartScreen unsigned,
   Wayland unsupported, UIPI, Win/Linux untested on device) — ask the user where (README?).
2. **Rust README** at repo root: what it is, install per OS (incl. Gatekeeper / SmartScreen
   steps, macOS permissions), build/run (`cargo run -p open-camera-mouse`), package
   (`cargo packager --release` + tools), release (tag `vX.Y.Z` = workspace version).
3. **Doc tidy:** stale docs and dead dev code (e.g. debug-only `OCM_DEV_*` env vars: keep or
   drop — ask).
4. **`tasks/` cleanup:** ask the user how (archive / condense / delete); keep Phase 8 open
   items somewhere (open risks in CURRENT_STATE, TEST_DRIVE #7).
5. **Remove the rewrite tooling, item by item with the user's OK:** `docs/archive/`
   (AGENTS.md-protected), `docs/research/tools/`, `TEST_DRIVE_RESEARCH_PROMPT.md`, RE /
   legacy-comparison research docs, `agent/`, `.agents/skills/`, `tasks/`,
   `scripts/record-scenario.sh`, rewrite-only parts of `ROADMAP.md`. Keep what build, CI and
   packaging need (`check-core-deps.sh`, `macos-adhoc-sign.sh`, `render-icon.py`, ADRs, QA
   docs — confirm).
6. **Remove `legacy/`** (user's explicit decision 2026-11-19): ADR recording it (Go app lives
   on in `main` history and the `v0.3.0` tag), then `.gitignore` legacy patterns, AGENTS.md /
   CLAUDE.md "Never" rule and map rows. Then slim AGENTS.md / CLAUDE.md to a post-rewrite
   version.
7. Gates: fmt, clippy, scoped tests (one heavy command at a time), CI green on 3 OS.

**After cleanup (user does it):** merge `rewrite/rust` → `main` — conflict on
`.github/workflows/release.yml` (legacy Go one on `main`): take the Rust one; tag `v0.4.0`
on `main`. That release run is the first time the Windows/Linux packaging executes (user: no
separate dry run) — check its jobs via the API and fix what fails, then the user publishes
the draft release.

**User decisions (Phase 7):** macOS Apple Silicon only, ad-hoc signed dmg, notarization hook
behind secrets (off); Windows unsigned NSIS per-user installer; Linux tar.gz + AppImage, X11
only, Wayland documented; GitHub Releases; version 0.4.0; icon = the user's solid-camera design (ADR-0021 icon amendment).

**Rules (ADR-0019/-0020):** measure first; changed defaults need the user's OK + ADR
amendment; every fix gets a CI test / replay metric / `OCM_PERF` + device check; recordings
diagnosis-only; one slice per session.

**Machine load:** one heavy command at a time, scoped (`-p`, test filter), say so first;
release builds: ask first.

**How the user runs the app:** `export PATH="$HOME/.cargo/bin:$PATH"` then
`cargo run -p open-camera-mouse` (dev build), or the `.app` from `target/packager/`.

**Known, not bugs:** Searching animation ignores Reduce Motion (Phase 8); font egui default;
F10 not F11 on macOS; re-detection off by default; no auto-recenter;
Gatekeeper rejects the ad-hoc app (documented in slice 8).

**User direction (don't drift):** start from 2018 but improve; new behaviour as options, off by
default unless approved; monochrome UI, neon green ring marker, yellow primary buttons,
Carbon/Paper, two fixed windows; hotkeys fixed (F10/F12 on macOS); automated checks for every
fix; real numbers on controls.

**Toolchain:** `export PATH="$HOME/.cargo/bin:$PATH"`. No `gh`; CI via the GitHub API with curl
(`.../actions/runs/<run>/jobs`, `.../check-runs/<job id>/annotations`).

**Blockers:** none. No Windows/Linux machines (slices 5–6 deferred, user 2026-11-19).
