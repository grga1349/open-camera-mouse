package hotkeys

import "errors"

// Action represents a hotkey callback.
type Action func()

// Manager listens for global hotkeys.
type Manager interface {
	Update(map[string]Action) error
	Close()
}

var ErrUnsupported = errors.New("hotkeys: global shortcuts unsupported on this platform")

// NewManager builds a platform-specific manager.
func NewManager() (Manager, error) {
	return newManager()
}
