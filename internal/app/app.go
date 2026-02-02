package app

import (
	"context"
	"errors"
	"image"
	"sync"
	"time"

	"open-camera-mouse/internal/camera"
	"open-camera-mouse/internal/config"
	"open-camera-mouse/internal/mouse"
	"open-camera-mouse/internal/stream"
	"open-camera-mouse/internal/tracking"
)

var (
	ErrAlreadyRunning = errors.New("app: capture already running")
	ErrNotRunning     = errors.New("app: capture not running")
	ErrNoFrame        = errors.New("app: no frame available")
)

const (
	previewInterval = 66 * time.Millisecond // ~15 fps
)

// Service coordinates the camera, frame processor, and cursor mover.
type Service struct {
	cfgManager   *config.Manager
	notifyParams func(config.AllParams)

	mu     sync.RWMutex
	params config.AllParams

	camera         *camera.Manager
	frameProcessor *FrameProcessor
	cursorMover    *CursorMover

	ctx     context.Context
	cancel  context.CancelFunc
	running bool
}

// NewService creates a new application service.
func NewService(cfg *config.Manager, notify func(config.AllParams)) (*Service, error) {
	params, err := cfg.Load()
	if err != nil {
		return nil, err
	}

	// Apply dwell on startup preference
	if !params.General.DwellOnStartup {
		params.Clicking.DwellEnabled = false
	}

	broker := stream.NewBroker()
	controller := mouse.NewRobotController()

	svc := &Service{
		cfgManager:   cfg,
		notifyParams: notify,
		params:       params,
		camera:       camera.NewManager(0),
	}

	// Create frame processor
	svc.frameProcessor = NewFrameProcessor(
		buildTrackerParams(params.Tracking),
		buildProcessorParams(params.Tracking),
		previewInterval,
		broker,
	)

	// Create cursor mover with dwell callback
	svc.cursorMover = NewCursorMover(
		controller,
		buildMappingParams(params.Pointer),
		buildDwellParams(params.Clicking),
		svc.handleDwellClick,
	)

	return svc, nil
}

// Broker returns the stream broker for subscribing to events.
func (s *Service) Broker() *stream.Broker {
	return s.frameProcessor.Broker()
}

// Start begins camera capture and processing.
func (s *Service) Start(ctx context.Context) error {
	s.mu.Lock()
	if s.running {
		s.mu.Unlock()
		return ErrAlreadyRunning
	}

	captureCtx, cancel := context.WithCancel(ctx)
	s.ctx = captureCtx
	s.cancel = cancel
	s.running = true
	s.mu.Unlock()

	if err := s.camera.Start(captureCtx, s.handleFrame); err != nil {
		s.mu.Lock()
		s.running = false
		s.cancel = nil
		s.mu.Unlock()
		return err
	}

	return nil
}

// Stop stops camera capture and processing.
func (s *Service) Stop() error {
	s.mu.Lock()
	if !s.running {
		s.mu.Unlock()
		return ErrNotRunning
	}
	if s.cancel != nil {
		s.cancel()
	}
	s.running = false
	s.mu.Unlock()

	s.camera.Stop()
	s.frameProcessor.Close()
	s.cursorMover.Reset()

	return nil
}

// IsRunning returns whether the service is currently running.
func (s *Service) IsRunning() bool {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.running
}

// handleFrame is called for each camera frame.
func (s *Service) handleFrame(frame camera.Frame) {
	// Process frame (tracking, overlay, preview)
	result := s.frameProcessor.Process(frame)

	// Update cursor position
	s.cursorMover.Update(result.Point, result.Lost)

	// Update dwell click state
	s.cursorMover.UpdateDwell(result.Lost)
}

// handleDwellClick is called when a dwell click occurs.
func (s *Service) handleDwellClick() {
	s.mu.Lock()
	defer s.mu.Unlock()

	if s.params.Clicking.RightClickToggle {
		s.params.Clicking.RightClickToggle = false
		s.applyRuntimeParamsLocked()
		s.emitParamsLocked()
	}
}

// SetPickPoint sets a new tracking template at the given point.
func (s *Service) SetPickPoint(point image.Point) error {
	err := s.frameProcessor.SetPickPoint(point)
	if err == nil {
		s.cursorMover.Reset()
	}
	return err
}

// Recenter resets tracking to the center of the frame and centers the cursor.
func (s *Service) Recenter() error {
	err := s.frameProcessor.Recenter()
	if err == nil {
		s.cursorMover.Reset()
		s.cursorMover.CenterCursor()
	}
	return err
}

// ToggleTracking enables or disables tracking.
func (s *Service) ToggleTracking(enabled bool) {
	s.frameProcessor.SetTrackingEnabled(enabled)
	if !enabled {
		s.cursorMover.Reset()
	}
}

// GetParams returns the current parameters.
func (s *Service) GetParams() config.AllParams {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.params
}

// UpdateParams updates the runtime parameters.
func (s *Service) UpdateParams(next config.AllParams) {
	s.mu.Lock()
	s.params = next
	s.applyRuntimeParamsLocked()
	s.emitParamsLocked()
	s.mu.Unlock()
}

// SaveParams saves parameters to disk.
func (s *Service) SaveParams(next config.AllParams) error {
	s.UpdateParams(next)
	return s.cfgManager.Save(next)
}

func (s *Service) emitParamsLocked() {
	if s.notifyParams == nil {
		return
	}
	params := s.params
	go s.notifyParams(params)
}

func (s *Service) applyRuntimeParamsLocked() {
	// Update tracker params
	s.frameProcessor.SetTrackerParams(buildTrackerParams(s.params.Tracking))
	s.frameProcessor.SetProcessorParams(buildProcessorParams(s.params.Tracking))

	// Update cursor mover params
	s.cursorMover.SetMappingParams(buildMappingParams(s.params.Pointer))
	s.cursorMover.SetDwellParams(buildDwellParams(s.params.Clicking))
}

// Helper functions to build component params from config

func buildTrackerParams(t config.TrackingParams) tracking.Params {
	return tracking.Params{
		TemplateSize:     t.TemplateSizePx,
		SearchMargin:     t.SearchMarginPx,
		ScoreThreshold:   float32(t.ScoreThreshold),
		AdaptiveTemplate: t.AdaptiveTemplate,
		TemplateAlpha:    float32(t.TemplateUpdateAlpha),
	}
}

func buildProcessorParams(t config.TrackingParams) FrameProcessorParams {
	return FrameProcessorParams{
		MarkerShape:  string(t.MarkerShape),
		TemplateSize: t.TemplateSizePx,
	}
}

func buildMappingParams(p config.PointerParams) mouse.MappingParams {
	gain := mapRange(float64(p.Sensitivity), 1, 100, 1.2, 5.0) * 4
	smoothing := mapRange(float64(p.Sensitivity), 1, 100, 0.35, 0.15)
	gainX := gain
	gainY := gain

	if adv := p.Advanced; adv != nil {
		if adv.GainX != 0 {
			gainX = adv.GainX
		}
		if adv.GainY != 0 {
			gainY = adv.GainY
		}
		if adv.Smoothing != 0 {
			smoothing = adv.Smoothing
		}
	}

	return mouse.MappingParams{
		Sensitivity: float64(p.Sensitivity),
		GainX:       gainX,
		GainY:       gainY,
		Smoothing:   smoothing,
		DeadzonePx:  max(0, float64(p.DeadzonePx)),
		MaxSpeedPx:  max(1, float64(p.MaxSpeedPx)),
	}
}

func buildDwellParams(c config.ClickingParams) mouse.DwellParams {
	return mouse.DwellParams{
		Enabled:     c.DwellEnabled,
		DwellTime:   time.Duration(c.DwellTimeMs) * time.Millisecond,
		RadiusPx:    float64(c.DwellRadiusPx),
		ClickButton: mapClickButton(c.ClickType, c.RightClickToggle),
	}
}

func mapClickButton(click config.ClickType, rightToggle bool) mouse.ClickButton {
	if rightToggle {
		return mouse.ClickRight
	}
	switch click {
	case config.ClickTypeRight:
		return mouse.ClickRight
	case config.ClickTypeDouble:
		return mouse.ClickLeft // TODO: implement double click
	default:
		return mouse.ClickLeft
	}
}

func mapRange(value, inMin, inMax, outMin, outMax float64) float64 {
	if inMax == inMin {
		return outMin
	}
	if value < inMin {
		value = inMin
	}
	if value > inMax {
		value = inMax
	}
	return (value-inMin)*(outMax-outMin)/(inMax-inMin) + outMin
}
