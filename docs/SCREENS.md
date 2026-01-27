# Screens — Wails v2 Single-Window UX
## MainScreen (operational)
### Must-have UI
- Top: title + status (OK/LOST) + score + fps
- Camera preview box:
- click-to-pick tracking point
- overlay marker (circle/square)
- overlay score + LOST indicator
- Controls:
- Recenter button (primary)
- Start/Pause toggle
- Auto click (dwell) checkbox
- Right click toggle
- Settings button → SettingsScreen


### Behavior
- MainScreen changes apply immediately for:
- Recenter
- Start/Pause
- Dwell enable toggle
- Right click toggle


## SettingsScreen (tabs + sticky actions)
### Header
- Back button
- “Settings” title


### Tabs
- Tracking
- Pointer
- Clicking
- Hotkeys (MVP: focus-only; display planned hotkeys)


### Sticky bottom bar
- Cancel (discard draft + return)
- Save (persist + apply + return)


### Draft model
- On enter: snapshot=current params, draft=copy(snapshot)
- UI edits draft only
- Cancel: discard draft
- Save: call backend SaveParams(draft)


## Settings tab contents (MVP)
### Tracking tab
- Template size (16/20/24)
- Search margin slider
- Score threshold slider
- Adaptive template toggle
- Template update alpha slider (enabled only if adaptive)
- Marker shape (circle/square)


### Pointer tab
- Sensitivity slider (master)
- Advanced (optional collapsible): gainX, gainY, smoothing
- Deadzone
- Max speed


### Clicking tab
- Dwell enabled
- Dwell time
- Dwell radius
- Click type (left/right/double)
- Right click toggle


### Hotkeys tab
- Show:
- Recenter hotkey (focus-only)
- Start/Pause hotkey (focus-only)
- MVP: implement via app menu accelerators or in-app key handlers while focused
