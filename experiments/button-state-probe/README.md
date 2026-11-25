# A3 macOS button-state probe

This temporary experiment checks whether `CGEventSourceButtonState(HIDSystemState)` sees a left mouse down that Open Camera Mouse could post. It uses the app's Quartz source state, zero suppression interval and HID posting location. The probe waits 750 ms before releasing so that it can sample the hold. A source tag lets its own AppKit window confirm delivery. It also logs `CombinedSessionState` for comparison.

## Run during the A6 device check

1. Quit Open Camera Mouse and any other program posting mouse events. Use a local macOS desktop session. Open Terminal and run the following from the repository root. The build uses the SDK and Command Line Tools.

   ```sh
   probe_dir="$(mktemp -d "${TMPDIR:-/tmp}/ocm-button-state-probe.XXXXXX")"
   xcrun clang -fobjc-arc -Wall -Wextra -Werror \
     -framework Cocoa -framework ApplicationServices \
     experiments/button-state-probe/probe.m -o "$probe_dir/probe"
   "$probe_dir/probe" --run
   ```

2. If the probe reports denied posting access, enable the launching terminal in System Settings → Privacy & Security → Accessibility. Reopen the terminal and repeat step 1. A permission check can pass even when delivery fails. The probe uses its own window to verify delivery and requires no Input Monitoring grant.
3. In the probe window, press and hold the physical left mouse button for a full second. Release it while the pointer is still within the window. This checks that the HID query actually sees a hardware press on this machine.
4. Leave the pointer near the center of the probe window. Press Space once. Keep your hands off the mouse or trackpad and leave the probe window active until it says it has finished. It checks that the pointer is over the probe's visible content and then posts one down/up pair at that location. The window has no action that a click can trigger.
5. Close the probe window and copy the terminal output from the macOS version through the verdict. Report it for A3/A7. Repeat the run from the same terminal if the result is inconclusive.

## Interpret the result

- **INCLUDES** means the HID query reported down after the window received the tagged down and before the probe posted the tagged up. The physical control passed and the button state returned to up.
- **EXCLUDES IN OBSERVED SAMPLES** means the physical control passed, the window received both tagged events, the HID query stayed up in repeated samples during the confirmed synthetic hold, and both state tables settled up after release. The result applies to the tested macOS build and posting setup.
- **INCONCLUSIVE** means the controls did not separate state from delivery or interference. A missing tagged event may indicate Accessibility or delivery trouble. Keep the window active and the pointer still on a repeat. If the physical control fails, retry the full one-second press. Keep the complete log if the failure persists.

Leave the process running until the result appears so that the scheduled up can run. If the process is interrupted during the hold, physically press and release the left button inside the blank window or over an inert part of the desktop to clear the held state. Afterward, remove the temporary executable with `rm "$probe_dir/probe"` and its directory with `rmdir "$probe_dir"`. If you enabled Accessibility just for this check, restore its previous setting.
