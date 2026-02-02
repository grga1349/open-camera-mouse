package hotkeys

import "errors"

type Action func()

type Manager interface {
	Update(map[string]Action) error
	Close()
}

var ErrUnsupported = errors.New("hotkeys: global shortcuts unsupported on this platform")

func NewManager() (Manager, error) {
	return newManager()
}
