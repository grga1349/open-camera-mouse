# Open Camera Mouse

Hands-free desktop application that turns your webcam into a mouse controller by tracking a
user-selected point in the camera image.

## Status: Rust rewrite in progress (Phase 0)

This branch (`rewrite/rust`) is rebuilding the product from scratch in Rust, dropping the
OpenCV/GoCV dependency. The previously shipped, fully working **v0.3.0 Go/Wails app lives
under [`legacy/`](legacy/)** — see `legacy/README.md` to build and run it; it is not touched
by work on this branch.

For AI agents (Claude Code, Codex, or otherwise): **start at [`AGENTS.md`](AGENTS.md)**, not
this file.

## Where things stand

- Phase 0 (repository foundation) — see `tasks/active/0001-phase-0-foundation.md` and
  `agent/CURRENT_STATE.md` for the live status.
- No Rust code exists yet. There is no `Cargo.toml`. That's intentional — see
  `docs/archive/rust-rewrite-seed/02-roadmap-nine-phases.md` for the nine-phase plan this
  rewrite follows, and `docs/architecture/decisions/` for what's been decided vs. proposed.

## Map

| Path | What's there |
|---|---|
| `legacy/` | The working Go/Wails/GoCV app (reference and regression baseline) |
| `docs/product/` | Why this exists, goals, v1 scope |
| `docs/research/` | Camera Mouse 2018 RE evidence vs. `legacy/`'s actual tracker, labeled |
| `docs/architecture/decisions/` | ADRs |
| `docs/design/` | UI design plan (HTML design lab lands here in Phase 5) |
| `docs/archive/rust-rewrite-seed/` | Original planning seed, archived input — not live policy |
| `tasks/` | Task backlog and lifecycle |
| `agent/` | Durable session state (current state, handoff, workflow conventions) |
| `.agents/skills/` | Project-local skill catalog |

## License

MIT — see [`LICENSE`](LICENSE).
