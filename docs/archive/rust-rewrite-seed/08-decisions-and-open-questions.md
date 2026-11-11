# Decision register, open questions and risks

## Agreed intent (from conversation)

- Rust rewrite on a separate branch, leaving original code available under `legacy/`.
- No OpenCV dependency for the new tracking core; write and test own algorithms.
- Use actual Camera Mouse 2018 RE evidence and legacy app for research.
- Claude Code and Codex should both work against shared project instructions.
- Self-contained repository with local skills, subagents, docs, tasks and handoff context.
- Main coordinator can delegate; preserve a durable summary on disk.
- Explore interactive HTML design lab before committing to final egui UI.
- The **current `dump/` package is staging only**; agent later creates real repo files.

## Proposed, not yet ratified

- Branch name `rewrite/rust`.
- `AGENTS.md` canonical with `CLAUDE.md` symlink.
- Canonical `.agents/skills/` with `.claude/skills` symlink.
- 3 Rust crates (`tracking-core`, `pointer-core`, `platform`) plus desktop app.
- egui/eframe UI; camera backend to be researched.
- PyrLK as initial behavioral baseline and old matcher as comparator.
- Eight seed skills and several specialist agent roles.
- Markdown task queue and durable `CURRENT_STATE`/`HANDOFF`.

## Questions requiring evidence/choice

1. What are the exact paths/layout and working-tree state today?
2. Should release target Windows first, macOS first, or parity from the outset?
3. What are current tool versions and their real project-local skill/subagent/config capabilities?
4. Do symlinks work correctly for both agents, including nested directories and Windows clones?
5. Which camera backends are currently maintained, lightweight and legally compatible?
6. Which platform cursor API supports expected behavior with OS permissions and Wayland limits?
7. What is the old tracker algorithm and quality vs binary-researched PyrLK?
8. What are acceptance thresholds for tracking error, jitter, recovery and CPU usage?
9. How should webcam frames be stored/annotated for privacy-safe fixtures?
10. How should concurrent agents lock tasks and prevent file conflicts?
11. What is minimum viable config, packaging and accessibility on each platform?
12. Which proposed agent skills are truly needed versus duplicated built-in CLI tools?

## Risks

Unverified decompiler details; OpenCV version-specific behavior; optical-flow drift; lighting and occlusion; excessive runtime dependencies; FPS/latency mismatches; headless UI test limitations; macOS permissions and codesigning; Wayland pointer control; overengineering AI automation; secrets and machine-specific paths committed to repo; original binary licensing; stale documentation.

## Change control

A proposed decision becomes accepted only after source-based investigation, alternatives, tradeoffs and, where relevant, a small spike or benchmark. Record the choice in an ADR rather than mutating the seed silently.
