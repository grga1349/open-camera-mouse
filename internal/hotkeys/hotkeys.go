package hotkeys

import (
	"fmt"

	"golang.design/x/hotkey"
)

var namedKeys = map[string]hotkey.Key{
	"F1": hotkey.KeyF1, "F2": hotkey.KeyF2, "F3": hotkey.KeyF3, "F4": hotkey.KeyF4,
	"F5": hotkey.KeyF5, "F6": hotkey.KeyF6, "F7": hotkey.KeyF7, "F8": hotkey.KeyF8,
	"F9": hotkey.KeyF9, "F10": hotkey.KeyF10, "F11": hotkey.KeyF11, "F12": hotkey.KeyF12,
	"F13": hotkey.KeyF13, "F14": hotkey.KeyF14, "F15": hotkey.KeyF15, "F16": hotkey.KeyF16,
	"F17": hotkey.KeyF17, "F18": hotkey.KeyF18, "F19": hotkey.KeyF19, "F20": hotkey.KeyF20,
}

// ParseKey resolves a key name (e.g. "F11") to a hotkey.Key. Only F1-F20 are
// supported, matching what golang.design/x/hotkey can register without
// modifiers across macOS, Windows, and Linux.
func ParseKey(name string) (hotkey.Key, bool) {
	k, ok := namedKeys[name]
	return k, ok
}

type Hotkeys struct {
	startPause   *hotkey.Hotkey
	recenter     *hotkey.Hotkey
	onStartPause func()
	onRecenter   func()
}

func Start(startPauseKey, recenterKey string, onStartPause func(), onRecenter func()) (*Hotkeys, error) {
	h := &Hotkeys{onStartPause: onStartPause, onRecenter: onRecenter}
	if err := h.register(startPauseKey, recenterKey); err != nil {
		return nil, err
	}
	return h, nil
}

// SetKeys re-registers both hotkeys under new key names, replacing the
// previous registration only after the new one succeeds.
func (h *Hotkeys) SetKeys(startPauseKey, recenterKey string) error {
	prevStartPause, prevRecenter := h.startPause, h.recenter
	if err := h.register(startPauseKey, recenterKey); err != nil {
		return err
	}
	if prevStartPause != nil {
		prevStartPause.Unregister()
	}
	if prevRecenter != nil {
		prevRecenter.Unregister()
	}
	return nil
}

func (h *Hotkeys) register(startPauseKey, recenterKey string) error {
	if startPauseKey == recenterKey {
		return fmt.Errorf("hotkeys: start/pause and recenter must be different keys")
	}

	startKey, ok := ParseKey(startPauseKey)
	if !ok {
		return fmt.Errorf("hotkeys: invalid start/pause key %q", startPauseKey)
	}
	recenterKeyCode, ok := ParseKey(recenterKey)
	if !ok {
		return fmt.Errorf("hotkeys: invalid recenter key %q", recenterKey)
	}

	startPause := hotkey.New(nil, startKey)
	if err := startPause.Register(); err != nil {
		return fmt.Errorf("hotkeys: register %s: %w", startPauseKey, err)
	}

	recenter := hotkey.New(nil, recenterKeyCode)
	if err := recenter.Register(); err != nil {
		startPause.Unregister()
		return fmt.Errorf("hotkeys: register %s: %w", recenterKey, err)
	}

	h.startPause = startPause
	h.recenter = recenter

	go func() {
		for range startPause.Keydown() {
			h.onStartPause()
		}
	}()
	go func() {
		for range recenter.Keydown() {
			h.onRecenter()
		}
	}()

	return nil
}

func (h *Hotkeys) Stop() {
	if h == nil {
		return
	}
	if h.startPause != nil {
		h.startPause.Unregister()
	}
	if h.recenter != nil {
		h.recenter.Unregister()
	}
}
