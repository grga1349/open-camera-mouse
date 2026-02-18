package hotkeys

import (
	"errors"
	"fmt"
	"strings"
	"sync"

	"golang.design/x/hotkey"
)

type Action func()

type Manager interface {
	Update(map[string]Action) error
	Close()
}

var ErrUnsupported = errors.New("hotkeys: global shortcuts unsupported on this platform")

func NewManager() (Manager, error) {
	return &manager{}, nil
}

type entry struct {
	hk   *hotkey.Hotkey
	done chan struct{}
}

type manager struct {
	mu      sync.Mutex
	entries []*entry
}

func (m *manager) Update(bindings map[string]Action) error {
	m.mu.Lock()
	old := m.entries
	m.entries = nil
	m.mu.Unlock()

	for _, e := range old {
		e.hk.Unregister()
		<-e.done
	}

	var entries []*entry
	for keyStr, action := range bindings {
		keyStr = strings.TrimSpace(keyStr)
		if keyStr == "" || action == nil {
			continue
		}
		key, err := parseKey(keyStr)
		if err != nil {
			for _, e := range entries {
				e.hk.Unregister()
				<-e.done
			}
			return fmt.Errorf("hotkeys: %w", err)
		}
		hk := hotkey.New(nil, key)
		if err := hk.Register(); err != nil {
			for _, e := range entries {
				e.hk.Unregister()
				<-e.done
			}
			return fmt.Errorf("hotkeys: register %s: %w", keyStr, err)
		}
		done := make(chan struct{})
		entries = append(entries, &entry{hk: hk, done: done})
		go func(hk *hotkey.Hotkey, action Action, done chan struct{}) {
			defer close(done)
			for range hk.Keydown() {
				go action()
			}
		}(hk, action, done)
	}

	m.mu.Lock()
	m.entries = entries
	m.mu.Unlock()
	return nil
}

func (m *manager) Close() {
	m.mu.Lock()
	old := m.entries
	m.entries = nil
	m.mu.Unlock()

	for _, e := range old {
		e.hk.Unregister()
		<-e.done
	}
}

func parseKey(input string) (hotkey.Key, error) {
	key := strings.ToUpper(strings.ReplaceAll(input, " ", ""))
	if key == "" {
		return 0, fmt.Errorf("empty key")
	}
	k, ok := functionKeys[key]
	if !ok {
		return 0, fmt.Errorf("unsupported key %s", key)
	}
	return k, nil
}

var functionKeys = map[string]hotkey.Key{
	"F1":  hotkey.KeyF1,
	"F2":  hotkey.KeyF2,
	"F3":  hotkey.KeyF3,
	"F4":  hotkey.KeyF4,
	"F5":  hotkey.KeyF5,
	"F6":  hotkey.KeyF6,
	"F7":  hotkey.KeyF7,
	"F8":  hotkey.KeyF8,
	"F9":  hotkey.KeyF9,
	"F10": hotkey.KeyF10,
	"F11": hotkey.KeyF11,
	"F12": hotkey.KeyF12,
	"F13": hotkey.KeyF13,
	"F14": hotkey.KeyF14,
	"F15": hotkey.KeyF15,
	"F16": hotkey.KeyF16,
	"F17": hotkey.KeyF17,
	"F18": hotkey.KeyF18,
	"F19": hotkey.KeyF19,
	"F20": hotkey.KeyF20,
}
