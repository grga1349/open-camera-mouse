package app

import (
	"context"
	"errors"
	"image"
	"sync"

	"open-camera-mouse/internal/camera"
	"open-camera-mouse/internal/config"
	"open-camera-mouse/internal/cursor"
	"open-camera-mouse/internal/mouse"
	"open-camera-mouse/internal/stream"
	"open-camera-mouse/internal/tracking"
)

var (
	ErrAlreadyRunning = errors.New("app: capture already running")
	ErrNotRunning     = errors.New("app: capture not running")
)

type Service struct {
	cfgManager *config.Manager
	paramsCh   chan config.AllParams

	mu     sync.RWMutex
	params config.AllParams

	camera  *camera.Service
	tracker *tracking.Service
	cursor  *cursor.Service

	cancel context.CancelFunc
	done   <-chan struct{}

	running bool
}

func NewService(cfg *config.Manager) (*Service, error) {
	params, err := cfg.Load()
	if err != nil {
		return nil, err
	}

	if !params.General.DwellOnStartup {
		params.Clicking.DwellEnabled = false
	}

	controller := mouse.NewRobotController()

	svc := &Service{
		cfgManager: cfg,
		paramsCh:   make(chan config.AllParams, 1),
		params:     params,
		camera:     camera.NewService(0),
		tracker:    tracking.NewService(buildTrackerParams(params.Tracking)),
	}

	svc.cursor = cursor.NewService(
		controller,
		buildMappingParams(params.Pointer),
		buildDwellParams(params.Clicking),
		svc.handleDwellClick,
	)

	return svc, nil
}

func (s *Service) Start(ctx context.Context) (<-chan stream.PreviewFrame, <-chan stream.Telemetry, error) {
	s.mu.Lock()
	if s.running {
		s.mu.Unlock()
		return nil, nil, ErrAlreadyRunning
	}

	captureCtx, cancel := context.WithCancel(ctx)
	s.cancel = cancel
	s.running = true
	s.mu.Unlock()

	previewCh, telemCh, err := s.runPipeline(captureCtx)
	if err != nil {
		s.mu.Lock()
		s.running = false
		s.cancel = nil
		s.mu.Unlock()
		cancel()
		return nil, nil, err
	}

	return previewCh, telemCh, nil
}

func (s *Service) runPipeline(ctx context.Context) (<-chan stream.PreviewFrame, <-chan stream.Telemetry, error) {
	frames, err := s.camera.Stream(ctx)
	if err != nil {
		return nil, nil, err
	}
	results := s.tracker.Stream(ctx, frames)
	previewCh, telemCh, done := s.cursor.Run(ctx, results)
	s.done = done
	return previewCh, telemCh, nil
}

func (s *Service) Stop() error {
	s.mu.Lock()
	if !s.running {
		s.mu.Unlock()
		return ErrNotRunning
	}
	done := s.done
	if s.cancel != nil {
		s.cancel()
	}
	s.running = false
	s.mu.Unlock()

	if done != nil {
		<-done
	}
	s.cursor.Reset()
	return nil
}

func (s *Service) IsRunning() bool {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.running
}

func (s *Service) ParamChanges() <-chan config.AllParams {
	return s.paramsCh
}

func (s *Service) handleDwellClick() {
	s.mu.Lock()
	defer s.mu.Unlock()

	if s.params.Clicking.RightClickToggle {
		s.params.Clicking.RightClickToggle = false
		s.applyRuntimeParamsLocked()
		s.emitParamsLocked()
	}
}

func (s *Service) SetPickPoint(point image.Point) error {
	err := s.tracker.SetPickPoint(point)
	if err == nil {
		s.cursor.Reset()
	}
	return err
}

func (s *Service) Recenter() error {
	err := s.tracker.Recenter()
	if err == nil {
		s.cursor.Reset()
		s.cursor.CenterCursor()
	}
	return err
}

func (s *Service) ToggleTracking(enabled bool) {
	s.tracker.SetTrackingEnabled(enabled)
	if !enabled {
		s.cursor.Reset()
	}
}

func (s *Service) GetParams() config.AllParams {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.params
}

func (s *Service) UpdateParams(next config.AllParams) {
	s.mu.Lock()
	s.params = next
	s.applyRuntimeParamsLocked()
	s.emitParamsLocked()
	s.mu.Unlock()
}

func (s *Service) SaveParams(next config.AllParams) error {
	s.UpdateParams(next)
	return s.cfgManager.Save(next)
}

func (s *Service) emitParamsLocked() {
	p := s.params
	select {
	case s.paramsCh <- p:
	default:
	}
}

func (s *Service) applyRuntimeParamsLocked() {
	s.tracker.UpdateParams(buildTrackerParams(s.params.Tracking))
	s.cursor.SetMappingParams(buildMappingParams(s.params.Pointer))
	s.cursor.SetDwellParams(buildDwellParams(s.params.Clicking))
}
