---
name: systematic-debugging
description: Root-cause method for bugs, flaky or failing tests, wrong cursor behaviour and device-only failures in Open Camera Mouse - reproduce, write hypotheses, collect evidence across the camera-to-cursor loop, change one variable, fix with a regression test, verify the original scenario. Use before proposing any fix to unexpected behaviour, and when a fix attempt failed. Not for reviewing a diff (code-review), test fixtures (tracking-validation, rust-testing) or performance baselines (benchmarking).
---

# Systematic debugging

## Objective

Find and prove the root cause before changing code, then fix it with an automated check that
failed first. This is `AGENTS.md` workflow step 3 ("measure first") made concrete. Guessing
fixes on a system that feeds its own output back in (the cursor we move is the cursor we
observe) produces new symptoms, not fixes.

## When to use / not to use

Use for: wrong or jumping cursor, misfiring clicks, tracker loss, misbehaving UI, failing or
flaky tests, CI-only failures, regressions, any failed fix attempt, and when the fix looks obvious.

Not for: reviewing a diff (`code-review`), test design (`rust-testing`,
`tracking-validation`), timing baselines (`benchmarking`). Roles are in `.agents/WORKFLOW.md`;
an investigator's report uses the labels below.

## The loop

1. **Reproduce.** Smallest deterministic repro: a failing test, a replay, an `OCM_PERF` or
   `OCM_INPUT` line, exact manual steps. Record OS, build (dev or release), settings, window,
   tab. If it cannot be reproduced, say so; gather more data, do not guess. Read the whole
   error, panic or log first.
2. **Check what changed.** `git log`, `git diff` against the last good tag or commit, setting
   or default changes (ADR amendments), dependency bumps. A regression window beats theory.
3. **Write the hypotheses down** (in the conversation, not new repo files): "H1: X is the
   cause because Y; if true, Z will be observed; if false, W." Keep two or three rivals alive.
   A hypothesis with no prediction cannot be tested.
4. **Collect evidence across boundaries.** Follow the data through each hop (map in
   `references/ocm-trace-map.md`) and log what enters and leaves the hop. Run once, find the
   first hop where input is right and output is wrong, then dig only there. Prefer existing
   diagnostics (`OCM_INPUT`, `OCM_PERF`, "Show diagnostics") over new `eprintln!`; remove
   temporary instrumentation before finishing.
5. **Test one variable.** One change per experiment; state what result confirms or kills
   which hypothesis. Disable one thing (a setting, a thread's effect, dwell), vary one input,
   or bisect commits. Never stack fixes. Experiments that need a device go to the user (below).
6. **Root cause, then fix.** You have a root cause when you can explain every observed fact
   and a test reproduces it. Fix at the source of the bad value, not where it shows up. No
   "while I'm here" edits; a changed default or behaviour needs the user (`AGENTS.md`).
7. **Regression test first.** Write it at the lowest layer that shows the bug, watch it fail
   for the right reason, then make it pass:

   Layer ladder (pure maths up to OS adapters): `references/ocm-trace-map.md`.

   If no layer can show it (OS delivery, real camera, permissions), say so and describe the
   manual check instead of inventing a mock that cannot fail.
8. **Verify the original scenario**, not just the new test: the repro steps from step 1, plus
   scoped gates from `AGENTS.md` (one heavy cargo command at a time). A device bug is not
   closed until the user has re-run it on the device; name the OS and say what is untested
   (Windows and Linux are often CI-only).

## Escalate, do not thrash

After **two** failed fix attempts, or when each fix moves the symptom elsewhere:
stop. Re-list the facts, check which assumption each failed fix relied on, and look for shared
state or feedback in the loop (a setting that changes the mapping, a cursor observation that
re-enters the controller). Report the facts, dead hypotheses and options (the design may be wrong); do not start a
third. If you do not know, say "I don't know X" and what would tell you.

Red flags: "just try", "probably X", a fix with no failing test, several changes in one run,
a theory that explains only some of the logs.

## Facts, hypotheses, conclusions

Label every statement in a report as **observed fact** (log, test output, `file:line`, user
report; the user saw something, not a cause), **hypothesis** (cause plus the prediction that
tests it) or **verified conclusion** (failing-then-passing test or experiment; say what was
not verified, e.g. an OS). Reading code is evidence for a hypothesis, only a failing test
for a conclusion. Example: `references/worked-example.md`.

## Device-only failures

When the failure needs a camera, OS cursor delivery or permissions, you cannot reproduce it.
Ask for the smallest run that discriminates between hypotheses: one scenario (about 2
minutes), exact steps, one diagnostic (`OCM_INPUT=1`, `OCM_PERF=1`, `OCM_RECORD` only for
tracker questions), what to send back. Convert every finding into a synthetic test; the log
is not the test. Check `pgrep` first (two instances fight over the cursor). Details:
`references/device-runs.md`.

## Deliverables and gates

A report with the labelled facts, hypotheses and conclusions, the failing-then-passing
test (or the reason none is possible) and the verification run with commands and outcomes,
plus the exact device check still owed. Gates: `AGENTS.md` Quality gates, scoped. Ask before
committing.

References (load when needed):

- `references/ocm-trace-map.md`: the camera-to-cursor loop hop by hop with what to log and
  which test layer covers each hop, the canonical test-layer ladder, plus loop-specific traps.
  Load for any pointer, click, handover or settings-changes-cursor bug.
- `references/device-runs.md`: diagnostics flags and how to ask for a device run.
- `references/worked-example.md`: facts vs hypotheses on a real slider-swing bug.

## Attribution

Process structure (reproduce, evidence at component boundaries, single hypothesis, one
variable, test before fix, stop after repeated failures) adapted from `systematic-debugging`
in obra/superpowers.
Source: https://github.com/obra/superpowers, `skills/systematic-debugging/SKILL.md`,
commit 22f0b463e7a5e579904dd6211a520321035526f7, licence MIT, Copyright (c) 2025 Jesse
Vincent. Adapted and rewritten for this repository; no upstream text copied verbatim beyond
short phrases. The MIT permission notice applies to the adapted material.
