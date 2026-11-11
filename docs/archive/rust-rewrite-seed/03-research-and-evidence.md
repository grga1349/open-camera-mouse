# Reverse engineering context — reported findings and verification checklist

## Source of report

A prior Codex investigation in sibling directory `../camera-mouse-reverse/` unpacked an Inno Setup Windows installer with `innoextract`, found the installed binary, and reportedly traced its 32-bit .NET Framework 4.0 managed code and bundled OpenCV libraries. It saved nine research documents including `research/TRACKING_SPEC.md`. The original installer itself was PE32 x86, which **alone did not establish the installed app runtime**; the subsequent analysis did.

## Reported behavior to VERIFY against research files

| Area | Reported observation | Caveat |
|---|---|---|
| Algorithm | OpenCV `cvCalcOpticalFlowPyrLK` | Callsite reported; native implementation not fully traced |
| Points | A single floating point selected in preview | Check exact coordinate transform |
| Frame format | Default tracking frames 8-bit single channel; conversion code `6` | Confirm color-conversion context and OpenCV version |
| LK window | `10×10` | Validate interpretation of parameter and source |
| Pyramid maximum level | `3` | Check actual pyramid levels and flags |
| Termination | `20` iterations, epsilon `0.03` | Confirm flags/criteria |
| Rejected displacement | `>35` image pixels replaced with previous point | Check squared vs Euclidean and when invoked |
| Edge margin | `10` pixels | Determine exact loss handling |
| Template history | Frame and pyramid buffers swapped; no separately fixed template shown | Absence of evidence, not proof of complete absence |
| Error signal | Null pointer passed for LK error output | No per-frame LK error read at that callsite; other confidence logic may exist |
| Mapping | Center-relative image position scaled to desktop with x/y sensitivity and offsets | Recover exact formula including dimensions/clamps |
| Default gains | x=6, y=6 | Confirm settings initialization and units |
| Cursor smoothing | `damping*mapped + (1-damping)*previous` | Verify implementation, especially naming of damping |
| Default damping | approximately `0.95` | May imply relatively little smoothing; do not infer empirically |
| Cursor output | WindowsInput absolute position normalized to `0..65535` | Check screens/multimonitor handling |
| Worker sleep | `70 ms` on main cursor loop paths | Do not equate to exact effective frame rate without runtime measurement |

## Known unresolved questions

Exact native OpenCV implementation and flags; initialization/preprocessing edge cases; tracking-loss countdown; reacquisition logic; dwell-click state machine; performance and live FPS; input device latency; real-world quality relative to current legacy implementation. Hopper Demo reportedly timed out opening native DLL, but no general Hopper limitation was established.

## How to use research correctly

- Read each file and record source paths and addresses for claims.
- Label `VERIFIED` (binary evidence), `INFERRED` (strong indirect evidence), `HYPOTHESIS`, `MEASURED` (benchmark) distinctly.
- Keep algorithm identification and algorithm quality claims separate.
- Mathematical LK outline: minimize intensity-constancy residual in a local window at successive pyramid scales, solve local normal equations iteratively; document gradient/interpolation/border details only when corroborated.
- Design clean Rust reimplementation from externally observable behavior, open literature and self-authored specifications; don't paste long proprietary decompiler output into shipped source.
- Original executable should be kept outside active app build and not distributed by the rewrite without rights.

## Benchmarks we want

Synthetic translation at subpixel and integer offsets; brightness shifts; local contrast changes; partial occlusion; head rotation/scale; motion blur; target near edge; lost-frame gaps; camera noise; fast moves; frame stalls. Track endpoint error, tracking-success ratio, time to reacquire, cursor jitter, frame-processing p50/p95/p99, allocations and CPU/memory.
