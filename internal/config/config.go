package config

import (
	"encoding/json"
	"errors"
	"os"
	"path/filepath"
)

const fileName = "config.json"

var ErrInvalidDir = errors.New("config: could not determine user config dir")

type Manager struct {
	path string
}

func NewManager(appName string) (*Manager, error) {
	dir, err := os.UserConfigDir()
	if err != nil {
		return nil, ErrInvalidDir
	}

	configDir := filepath.Join(dir, appName)
	if err := os.MkdirAll(configDir, 0o755); err != nil {
		return nil, err
	}

	return &Manager{path: filepath.Join(configDir, fileName)}, nil
}

func (m *Manager) Path() string {
	return m.path
}

func (m *Manager) Load() (AllParams, error) {
	if _, err := os.Stat(m.path); errors.Is(err, os.ErrNotExist) {
		return DefaultParams(), nil
	}

	data, err := os.ReadFile(m.path)
	if err != nil {
		return DefaultParams(), err
	}

	var params AllParams
	if err := json.Unmarshal(data, &params); err != nil {
		return DefaultParams(), err
	}

	return params, nil
}

func (m *Manager) Save(params AllParams) error {
	data, err := json.MarshalIndent(params, "", "  ")
	if err != nil {
		return err
	}

	return os.WriteFile(m.path, data, 0o644)
}
