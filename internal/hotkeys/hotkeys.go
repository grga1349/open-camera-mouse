package hotkeys

import (
	"fmt"

	"golang.design/x/hotkey"
)

type Hotkeys struct {
	startStop *hotkey.Hotkey
	recenter  *hotkey.Hotkey
}

func Start(onStartStop func(), onRecenter func()) (*Hotkeys, error) {
	h := &Hotkeys{}

	h.startStop = hotkey.New(nil, hotkey.KeyF11)
	h.recenter = hotkey.New(nil, hotkey.KeyF12)

	if err := h.startStop.Register(); err != nil {
		return nil, fmt.Errorf("hotkeys: register F11: %w", err)
	}

	if err := h.recenter.Register(); err != nil {
		h.startStop.Unregister()
		return nil, fmt.Errorf("hotkeys: register F12: %w", err)
	}

	go func() {
		for range h.startStop.Keydown() {
			onStartStop()
		}
	}()

	go func() {
		for range h.recenter.Keydown() {
			onRecenter()
		}
	}()

	return h, nil
}

func (h *Hotkeys) Stop() {
	if h == nil {
		return
	}
	if h.startStop != nil {
		h.startStop.Unregister()
	}
	if h.recenter != nil {
		h.recenter.Unregister()
	}
}
