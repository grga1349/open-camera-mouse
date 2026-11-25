# Planning: how work is organised and tracked

How the main agent plans, tracks and closes work that spans more than one session (ADR-0025).
`AGENTS.md` is the entry point and wins on conflict; how work is *executed and delegated* is
in [`WORKFLOW.md`](WORKFLOW.md); techniques for *writing and reviewing* a plan are in the
`engineering-planning` skill; worked examples are in its `references/examples.md`. Scale it:
a one-session task keeps its plan in the conversation or the PR and uses none of this.

## Where state lives

| State | Lives in | Owned by |
|---|---|---|
| Wishlist, deferred work, known limitations | `docs/ROADMAP.md` (permanent) | main agent, on the user's decisions |
| Plan, items, statuses, decisions, leftovers of one approved initiative | active roadmap `docs/<NAME>_ROADMAP.md` (temporary) | main agent |
| Resume point and evidence not yet placed elsewhere | active handoff `docs/<NAME>_HANDOFF.md` (temporary) | main agent |
| Durable decisions | ADRs, `docs/*.md`, tests | main agent, user approves |
| A delegated task | the delegation message and its report | main agent |
| History | git | — |

Each fact has one home; the others link to it. No other planning files: no per-item or
per-plan files, no task directories, no JSON/database state, no scripts.

## Levels

Product backlog (`docs/ROADMAP.md`) → active initiative or release roadmap → phases
(stretches of work in execution order) → work items (each with a priority) → delegated tasks. Delegated tasks never become
roadmap entries unless they turn into independent work worth tracking. **No entry authorises
work by itself:** an approved roadmap item is authorised only when the user asks for it (`rhmo`
or an explicit request); never start the next item by yourself (`WORKFLOW.md`). A request to
plan or review a plan is not one.

## Lifecycle of a multi-session initiative

1. **Seed:** the user's brief; its items are symptoms and leads, not diagnoses.
2. **Discovery:** read code, docs, ADRs; measure first; bounded investigations may be
   delegated. Record what is fact and what is hypothesis.
3. **Planning** (main context, strongest model): scope and exclusions, items with priorities,
   acceptance criteria and verification, dependencies, phases in execution order, the
   decisions table for what only the user can answer.
4. **Plan review:** for substantial or risky plans, an independent reviewer (strong tier),
   per the skill's `references/plan-review.md`.
5. **Approval:** the user approves the plan; only then create the roadmap, the handoff and
   the `AGENTS.md` "Read" row, on the initiative's branch.
6. **Execution:** one item (or a slice the user names) per session through `WORKFLOW.md`; replan on evidence.
7. **Integration:** each item lands with its tests; statuses updated in the same commit.
8. **Reconciliation** of everything not done (below).
9. **Final validation:** gates on all OSes in CI, device checks the release requires.
10. **Cleanup:** move what lasts to the permanent docs, ADRs and `docs/ROADMAP.md`; delete
    the roadmap, the handoff and the "Read" row before the merge.

## Active roadmap layout

The document reads in the order the work is done: what is next is at the top of the plan, and
reading it top to bottom is the critical path. Sections, in this order:

1. Header (purpose, branch, source, "deleted before merge"), rules for this initiative,
   legend.
2. **Decisions** table (`Q1`…, never clashing with a Phase D item: question, what it
   blocks, options and recommendation, answer with who and when). Every open question for the user is a row here, including scope
   proposals ("F6 to Leftovers?"); there is no separate proposals section.
3. **Plan:** phases in execution order (`## Phase A — …`), each a stretch of work that ends
   at a checkpoint the user can see (a device run, a slice of fixes, the release). Inside a
   phase, items appear in the order they will be done. Priority is per item, not per phase.
   An `[i]` item sits in the phase where its owed check happens.
4. **Done:** one line per `[x]`, `[-]` item (outcome, where the evidence lives: test name,
   ADR, doc). Details live in tests, ADRs and git, not here.
5. **Leftovers**, last.

No session log (git's and the handoff's job), and no standing "proposed order": a reorder or
plan the agent proposes is approved, changed or rejected with the user in the session it is
raised, then the plan is rewritten to match. When the plan is rearranged, items are
renumbered in plan order (phase letter + number: `A1`, `A2`, `B1`…) so the ids read in
sequence; the handoff of that session lists old → new once, and the next rewrite drops it.

## Work items

One checkbox line, `**id title**` stating the outcome and its priority (`P0`…), then
indented fields. Every open item has `Accept` (observable criteria) and `Verify` (tests,
gates, device checks and on which OS); add the others only where they add something:
`Needs` (items, decisions), `Blocks release` (if it differs from the priority), `Evidence`,
`Out of scope`, `Remaining`/`Open`. A user device run that checks several items is an item
of its own (`A6 macOS user run: …`) with the exact steps and the ids it closes, placed where it
happens in the plan. Outcomes, not
file lists; leads and file references are evidence, not the plan. An item is small enough
for one session and testable on its own. Between rearrangements an id is kept when the item
is split (`A3` → `A3a`, `A3b`), deferred or done, and never reused. Ids are temporary: code,
tests, commit-independent docs and ADRs never cite them (describe the behaviour or name the
release instead), so renumbering breaks nothing outside the roadmap and handoff.

## Status

The checkbox marker is the status; the text says why. No status anywhere else.

| Marker | Status | The line must say |
|---|---|---|
| `[ ]` | open (ready when its `Needs` are met) | — |
| `[~]` | in progress | what is done and what remains |
| `[?]` | blocked | on what: `Q3`, `device: Windows`, `needs A7`, an external fact |
| `[i]` | implemented, verification owed | what is verified and what is owed (`owed: macOS device`) |
| `[x]` | done | — (all mandatory acceptance criteria met) |
| `[>]` | deferred | where it went: `→ Leftovers`, `→ docs/ROADMAP.md App 1` |
| `[-]` | dropped | why, and who decided |

`[i]` when the check can run and waits on someone (e.g. the user's device run); `[?] device: X`
only when no X device or environment is available.

Verification is reported per level: **automated** (tests and gates locally, CI on macOS,
Windows, Linux); **device** per OS (named); **input** (physical button vs. synthetic or
dwell); **camera** (built-in vs. external). Passing tests are not a device check, CI on
Windows is not "verified on Windows", and a check the agent could not run is reported as not
run. `[x]` needs every mandatory level; an optional level may be skipped only when the
limitation is written on the item and the user accepted it. Whatever an item's `Verify`
says, it also needs CI green on all three OSes, plus a device check for device-only
behaviour (`AGENTS.md` step 5).

## Priorities and release blockers

`P0` blocks the release; `P1` belongs in it but may move to Leftovers with the user's OK;
`P2` is nice to have. Priority is written on each item; execution order is the plan's order
(risk first; a P1 may precede a P0 when it unblocks it, with the reason in the item, or
when the user approved that order, noted in the phase).
Only the user makes or unmakes a release blocker.

## Changing the plan

Roadmaps are living plans. When evidence (a test, a measurement, a device result, a review
finding, a spike) or a user decision invalidates part of the plan, update the items,
dependencies and priorities in the same session, and move the handoff's resume point if it
pointed at changed work. Plain-language instructions are roadmap operations; apply them
without ceremony:

| The user says | Operation |
|---|---|
| "Skip this for now", "Move this to leftovers" | `[>]`, entry in Leftovers |
| "This is less important" / "now a release blocker" | change the priority; re-order; blockers per the user |
| "We cannot test this right now" | split the check into its own item: `[?] device: …`, or `[>]` if not release-critical |
| "Drop this entirely" | `[-]` with the reason; keep evidence that a permanent doc needs |
| "Put this on the future roadmap" | `[>] → docs/ROADMAP.md`, entry written there now |
| "Do the important part now, the rest later" | split by id; the remainder to Leftovers |

Ask only when the instruction is unclear or conflicts with a mandatory requirement (say
which). Significant plan changes (scope, phase order, a dropped P0/P1) get an independent
plan review when the risk warrants it.

## Partial completion and unavailable verification

Keep the verified part; split the rest into its own item (same id plus a letter) when that
helps; move the rest to Leftovers or `docs/ROADMAP.md`; adjust criteria and dependencies;
write the reason. Never mark the full scope done. When a check cannot be run here, say
exactly which of these holds: implemented; automated verification passed; device check
pending (which OS, input, camera); behaviour not verified. If the missing check is a release
requirement, the item stays blocking until the user accepts the limitation.

## Leftovers

The last section of a substantial roadmap; not a separate file. For work that will not be
done in the current plan: lower priority, unavailable device checks, non-critical bugs,
optional optimisations, deferred investigations, partial remainders, work waiting on
information, cost above value. Each entry keeps: original id and phase, reason, done so far,
remaining, evidence and verification so far, dependencies, `blocks release: yes/no`, proposed
disposition. Leftovers are not done, and evidence or decisions are never discarded silently.

## Authority

The agent may recommend postponing, splitting, dropping or reprioritising, with the evidence.
The user owns product scope, release blockers and direction; an explicit user defer or drop
is applied without re-confirmation unless it is unclear or breaks a mandatory requirement.
The agent never: defers hard work silently; reclassifies a failing mandatory test or check
as optional; downgrades a blocker to finish; hides unfinished work by moving it; removes
acceptance criteria to look complete; abandons approved work without a record; starts a
`docs/ROADMAP.md` item or another initiative's work on its own. Durable decisions go into
ADRs or permanent docs, not only into the temporary roadmap.

## Reconciliation and cleanup

Before an initiative or release is declared complete, list every item that is not `[x]`
(open, blocked, `[i]`, `[~]`, Leftovers) and propose one disposition each: **complete now**;
**reactivate as blocker**; **transfer** to `docs/ROADMAP.md`; **accept as known limitation**
(documented in README/`TESTING.md`/an ADR); **drop as obsolete**. Base it on priority, risk,
effort, dependencies, verification and requirements. The user approves the release-scope
decisions; nothing is implemented from Leftovers without approval. Before deleting the
temporary files, check that every remaining item, its evidence and every durable decision has
a permanent home; then delete them in the cleanup commit.

## Handoff

The handoff holds what the next session needs to resume and nothing the roadmap, docs or git
already hold: **mode** (standard, `mc`, spike); **active item** (id); **resume point** and
next action; **working tree** (uncommitted, what it contains); **recent findings** not yet in
an item; **verification** (gates run with results; checks owed); **waiting on the user**
(decisions by id, device runs); blockers. Rewrite it at each checkpoint (`h`), with the
roadmap in sync. Keep a short note of the last session or two; fold older sessions away,
moving any unique evidence into its roadmap item or a permanent doc. The resume point lives
under `## Resume here` (the heading `rhmo` reads). It never points at a `[?]`, `[>]` or `[-]`
item; if the user asks for deferred work, reactivate it first (`[>]` → `[ ]`, noting who and
when).

## `docs/ROADMAP.md`

The permanent backlog: future features, known limitations, technical debt, refactorings,
investigations, deferred optimisations, platform work, outstanding verification, ideas the
user has not approved. Entries are not commitments and authorise nothing. A transfer from an
initiative keeps the context and evidence, remaining work, requirement and ADR references,
and prerequisites; merges into an existing entry instead of duplicating it; and drops the
initiative's execution details (sessions, slices, statuses). Remove an entry when it lands.
It is not an archive or a session log.

## Subagents and spikes

The main agent owns the roadmap, the plan and the handoff. Subagents investigate, evaluate
feasibility, review plans, implement and propose; they do not change priorities, scope, the
roadmap or the handoff (`WORKFLOW.md` delegation contract). Their findings may trigger
replanning by the main agent. A spike ends in one of: a new item, a revised plan, a rejected
approach (recorded with the evidence), a deferred improvement, an accepted limitation; never
in authorisation to put its code in production.
