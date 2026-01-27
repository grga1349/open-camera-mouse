# Tech Stack & Conventions — Camera Mouse MVP

## Platform
- Desktop framework: **Wails v2**
- Target OS: Windows, macOS, Linux

---

## Backend (Go)

### Language
- Go 1.22+

### Computer Vision
- **GoCV (OpenCV bindings)**

### OS Input
- Mouse control via **robotgo** (or alternative behind interface)

### Architecture
- Modular packages:
  - camera
  - tracking
  - mouse
  - overlay
  - stream
  - config
  - app

### Concurrency Model
- Single coordinator goroutine
- Frame pipeline via channels
- No shared mutable state without synchronization

---

## Frontend (React)

### Framework
- React 18
- No Next.js
- No SSR

### Styling
- **Tailwind CSS**
- Minimal custom CSS

### State
- Simple hooks + local store
- No Redux / MobX / Zustand

### Routing
- Internal screen switching (no react-router)

---

## UI Design System

### Window
- Fixed size: 420 × 820
- Mobile-style vertical layout

### Theme
- Dark mode only (MVP)
- High contrast for accessibility

### Components
- Reusable UI primitives:
  - Toggle
  - Slider
  - Tabs
  - StickyActions

---

## Coding Conventions

### Go
- gofmt + golines
- No reflection-heavy abstractions
- Explicit, readable code over cleverness

### React
- Functional components only
- Hooks only (no class components)
- Prefer composition over inheritance

---

## Build & Tooling

- Node 20+
- Go 1.22+
- Wails CLI v2

---

## Out of Scope (by design)

- Redux / complex state frameworks
- CSS-in-JS
- Electron
- ML frameworks (for MVP)
