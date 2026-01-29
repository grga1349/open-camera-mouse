//go:build !darwin

package hotkeys

func newManager() (Manager, error) {
	return nil, ErrUnsupported
}
