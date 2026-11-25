# Worked example: facts vs hypotheses

A physical press on a speed slider swings the value 1 to 60 while pointer events land at x
from -753 to 954.

- **Observed fact** (user report and `OCM_INPUT` log): the swing, and the event positions.
  Source of those events: unknown.
- A first attempt (post camera moves as drags while a button is held) did not fix it: a failed
  attempt, so stop and re-trace.
- **Hypothesis** (review, "traced" by reading): changing gain rescales the mapping offset, so
  the cursor moves under the held slider, the slider reads the new x, and the value changes
  again: a feedback loop through the OS cursor. Prediction: the Dwell time slider (no gain) is
  fine, which the user saw.
- It stays a hypothesis until a controller-rig test that changes gain under camera control
  fails. Other unexplained symptoms (a click on the preview while tracking re-selects the
  target; a synthetic dwell click inside a physical-button drag releases the drag) stay
  separate; none is folded into the gain theory.
