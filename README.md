# Open Camera Mouse

Hands-free desktop application that turns your webcam into a mouse controller by tracking a
user-selected point in the camera image.

## Status: Rust rewrite in progress (Phase 2: workspace skeleton)

This branch (`rewrite/rust`) is rebuilding the product from scratch in Rust, dropping the
OpenCV/GoCV dependency. The previously shipped, fully working **v0.3.0 Go/Wails app lives
under [`legacy/`](legacy/)** — see `legacy/README.md` to build and run it; it is not touched
by work on this branch.

For AI agents (Claude Code, Codex, or otherwise): **start at [`AGENTS.md`](AGENTS.md)**, not
this file.

## Where things stand

- Phase status: [`docs/ROADMAP.md`](docs/ROADMAP.md). Live state: `agent/CURRENT_STATE.md`.
- The Cargo workspace is a skeleton: crate boundaries, frame type, absolute pointer mapping,
  controller states and an empty egui window. No tracking algorithm yet (Phase 3).

```sh
cargo test --workspace          # no webcam needed
cargo run -p open-camera-mouse  # empty desktop window
```

Toolchain is pinned in `rust-toolchain.toml` (rustup installs it on first use).

## Map

| Path | What's there |
|---|---|
| `legacy/` | The working Go/Wails/GoCV app (reference and regression baseline) |
| `docs/product/` | Why this exists, goals, v1 scope |
| `docs/research/` | Camera Mouse 2018 RE evidence vs. `legacy/`'s actual tracker, labeled |
| `crates/`, `apps/desktop/` | Rust workspace (ADR-0007) |
| `docs/architecture/` | ADRs, `DEPENDENCIES.md`, `PLATFORM_RISKS.md` |
| `docs/design/` | UI design plan (HTML design lab lands here in Phase 5) |
| `docs/archive/rust-rewrite-seed/` | Original planning seed, archived input — not live policy |
| `tasks/` | Task backlog and lifecycle |
| `agent/` | Durable session state (current state, handoff, workflow conventions) |
| `.agents/skills/` | Project-local skill catalog |

## License

MIT — see [`LICENSE`](LICENSE).
