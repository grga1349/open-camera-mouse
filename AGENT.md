# Agent Instructions — Camera Mouse MVP


---


## 6. Code Style Rules


### Go
- Prefer clarity over cleverness
- Explicit types over generics (unless required)
- No reflection-heavy abstractions
- No unnecessary interfaces


### React
- Functional components only
- Hooks only
- No Redux / MobX / Zustand
- Minimal dependencies


---


## 7. CV + Tracking Rules


- Never move cursor when tracker is in LOST state.
- Never update template while LOST.
- Clamp deltas before mapping.
- Always apply smoothing.


---


## 8. Forbidden Patterns


- Global singletons
- Hidden background goroutines
- Event storms
- Deep nested callbacks
- Magic numbers (all tunables must be params)


---


## 9. MVP Scope Enforcement


Allowed:
- Template matching tracker
- Dwell clicking
- Simple calibration via settings


Forbidden (until post-MVP):
- ML face landmarks
- Eye tracking
- Gesture systems
- Macro engines
- Multi-profile UI


---


## 10. Task Execution Rules


When executing a task from `docs/TASKS.md`:


1. Implement only the specified task.
2. Do not modify unrelated files.
3. Do not refactor working code.
4. Write code that compiles and runs.
5. Prefer minimal solutions.


---


## 11. Design Philosophy


> This project prioritizes **stability, predictability, and usability over novelty**.


If a change improves simplicity and reliability → good.
If it adds complexity without measurable benefit → reject.


---


## 12. Golden Rule


If in doubt:
Choose the simplest implementation that preserves correctness and stability.
