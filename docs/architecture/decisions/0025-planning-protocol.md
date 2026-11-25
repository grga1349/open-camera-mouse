# ADR-0025: Planning protocol — temporary initiative roadmap + handoff

Status: Accepted (user request 2026-11-25)
Date: 2026-11-25
Amends: ADR-0022 (prompt-driven workflow), ADR-0024 (orchestration)

## Context

The website story and v0.4.1 each used a temporary `docs/<NAME>_ROADMAP.md` +
`docs/<NAME>_HANDOFF.md`, deleted before merge, but no ADR covered them, and `AGENTS.md`
said "no task files or session logs" and "plan not in new repo files". The v0.4.1 handoff
grew into a ~28 KB chronological log; statuses were ambiguous (`[-]` meant dropped and
deferred; nothing separated implemented from verified); nothing defined leftovers or an
end-of-release reconciliation. The user asked for structured, adaptive planning without
bureaucracy.

## Options considered

1. Plan only in the conversation or PR: loses state across sessions.
2. A task framework, per-item files or JSON state: rejected by ADR-0022; overhead.
3. **One canonical protocol (`.agents/PLANNING.md`) plus an `engineering-planning` skill,
   using the existing temporary roadmap + handoff pair.** Chosen.

## Decision

- A multi-session initiative the user approved gets exactly one temporary roadmap and one
  handoff on its branch; both are deleted before merge, after leftovers reconciliation has
  moved remaining work, evidence and durable decisions to `docs/ROADMAP.md`, permanent docs
  or ADRs.
- The roadmap owns the plan and statuses; the handoff owns only resume state (bounded, not
  a log); `docs/ROADMAP.md` is the permanent backlog and authorises nothing.
- Status markers: `[ ]` `[~]` `[?]` `[i]` `[x]` `[>]` `[-]` (open, in progress, blocked,
  implemented with verification owed, done, deferred, dropped); verification is recorded per
  level (automated per OS, device per OS, physical vs synthetic input, camera).
- Release scope and blockers are the user's call; the agent recommends with evidence. New
  command `lr` (leftovers review, report only).

## Consequences

- Multi-session work is legitimate and bounded; handoffs stay short.
- One more file to read when planning (`PLANNING.md`, loaded for planning and tracking only).
- Existing v0.4.1 files move to the new format at the next `h`; scope and priorities only
  change with the user's OK.
