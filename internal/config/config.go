package config

import (
	"encoding/json"
	"log"
	"os"
	"path/filepath"

	"open-camera-mouse/internal/hotkeys"
)

const (
	DefaultTemplateSizePx = 45
	DefaultGainMultiplier = 8.0
	DefaultSmoothing      = 0.30
	DefaultDwellTimeMs    = 500
	DefaultStartPause     = "F11"
	DefaultRecenter       = "F12"
)

type Params struct {
	TemplateSizePx int     `json:"templateSizePx"`
	GainMultiplier float64 `json:"gainMultiplier"`
	Smoothing      float64 `json:"smoothing"`
	DwellEnabled   bool    `json:"dwellEnabled"`
	DwellTimeMs    int     `json:"dwellTimeMs"`
	AutoStart      bool    `json:"autoStart"`
	StartPause     string  `json:"startPause"`
	Recenter       string  `json:"recenter"`
}

func DefaultParams() Params {
	return Params{
		TemplateSizePx: DefaultTemplateSizePx,
		GainMultiplier: DefaultGainMultiplier,
		Smoothing:      DefaultSmoothing,
		DwellTimeMs:    DefaultDwellTimeMs,
		StartPause:     DefaultStartPause,
		Recenter:       DefaultRecenter,
	}
}

type Manager struct {
	path string
}

func NewManager(appName string) (*Manager, error) {
	dir, err := os.UserConfigDir()
	if err != nil {
		return nil, err
	}
	return &Manager{path: filepath.Join(dir, appName, "config.json")}, nil
}

func (m *Manager) Load() (Params, error) {
	data, err := os.ReadFile(m.path)
	if err != nil {
		if os.IsNotExist(err) {
			return DefaultParams(), nil
		}
		return DefaultParams(), err
	}
	p := DefaultParams()
	if err := json.Unmarshal(data, &p); err != nil {
		log.Printf("config: failed to parse %s, using defaults: %v", m.path, err)
		return DefaultParams(), nil
	}
	if p.TemplateSizePx <= 0 {
		p.TemplateSizePx = DefaultTemplateSizePx
	}
	if p.GainMultiplier <= 0 {
		p.GainMultiplier = DefaultGainMultiplier
	}
	if p.Smoothing <= 0 || p.Smoothing > 1 {
		p.Smoothing = DefaultSmoothing
	}
	if p.DwellTimeMs <= 0 {
		p.DwellTimeMs = DefaultDwellTimeMs
	}
	if _, ok := hotkeys.ParseKey(p.StartPause); !ok {
		p.StartPause = DefaultStartPause
	}
	if _, ok := hotkeys.ParseKey(p.Recenter); !ok {
		p.Recenter = DefaultRecenter
	}
	if p.StartPause == p.Recenter {
		p.StartPause = DefaultStartPause
		p.Recenter = DefaultRecenter
	}
	return p, nil
}

func (m *Manager) Save(p Params) error {
	if err := os.MkdirAll(filepath.Dir(m.path), 0755); err != nil {
		return err
	}
	data, err := json.MarshalIndent(p, "", "  ")
	if err != nil {
		return err
	}
	return os.WriteFile(m.path, data, 0644)
}
