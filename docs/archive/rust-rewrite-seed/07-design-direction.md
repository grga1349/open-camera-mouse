# UI design research and design lab proposal

## Intent

Design an elegant, professional, native-feeling accessibility utility for a webcam-driven pointer, not a gimmicky dashboard. Explore visual approaches in a standalone interactive HTML file **after** foundation and design research, then translate selected tokens/interactions to egui. This seed does not contain the final HTML.

## Candidate directions

A. Dark technical, understated: charcoal/slate, mint-teal accent, clear tracking indicator, minimal chrome.
B. Light native utility: bright background, strong contrast, subtle borders, compact but generous hit targets.
C. High-contrast accessibility: larger text/hit areas, explicit status, minimal reliance on color alone.

Illustrative (NOT decided) tokens for A: `--bg:#111820`, `--surface:#1d2933`, `--accent:#53d5bb`, `--text:#edf4f6`, `--muted:#8397a3`, `--radius:8px`. User should iterate and approve; don't hardcode seed colors as product requirements.

## Core UI elements

Camera preview with target reticle and selection; clear tracking/paused/lost/permission status; pause/resume; sensitivity X/Y; smoothing mode and amount; pointer limits and calibration; camera selector; dwell click settings; accessible onboarding/help; diagnostics toggles for testing.

## States to prototype

First run/no permission, no camera, preview running/unselected, target selected/tracking, temporarily lost, recovery, manually paused, camera disconnected, loading and error. Include indicators not relying solely on hue, keyboard navigation and meaningful contrast.

## HTML design lab

Propose `docs/design/design-lab.html` in the future **actual repo**. It should run locally with no backend, use CSS tokens, allow switching theme/component variants and simulated app states, and contain realistic component hierarchy rather than static marketing art. No production web app is needed; its purpose is fast iteration before egui.

## Handoff into egui

Review each visual affordance for immediate-mode feasibility, scaling, fonts, embedded resources, performance of preview updates, native window sizes and high-DPI behavior. Record token mapping to egui `Visuals`/`Style`, keeping accessibility priorities.
