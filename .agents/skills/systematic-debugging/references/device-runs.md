# Device runs

When the failure needs a camera, OS cursor delivery or permissions, the agent cannot
reproduce it. Ask for the smallest run that discriminates between the hypotheses: one
scenario (about 2 minutes), exact steps, one diagnostic, and what to send back.

- `OCM_INPUT=1`: per-window pointer, button, leave and focus log; for click, drag, slider and
  focus problems. If a log lacks what a hypothesis needs (active tab, OS event type), extend
  the log first, then ask for the run.
- `OCM_PERF=1`: frame-path timing and tracker status counts; for lag, fps, loss.
- `OCM_RECORD=<folder>`: raw frames, only when the tracker itself is in question. Recordings
  and numbers from them are diagnosis only, kept outside the repo, never committed.
- Convert every finding into a synthetic test (kittest replay of the logged events, controller
  rig sequence, fake pointer, synthetic fixture) so CI keeps it; the log itself is not the test.
- Ask one question at a time that separates hypotheses (for example "was the first click on a
  speed slider, and was the Options window left of centre?").
- Check `pgrep` for a running app instance before a device run; two instances fight over the
  cursor.

