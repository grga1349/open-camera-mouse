package app

import (
	"context"
	"errors"
	"image"
	"image/color"
	"sync"
	"time"

	"open-camera-mouse/internal/camera"
	"open-camera-mouse/internal/config"
	"open-camera-mouse/internal/mouse"
	"open-camera-mouse/internal/overlay"
	"open-camera-mouse/internal/stream"
	"open-camera-mouse/internal/tracking"

	"gocv.io/x/gocv"
)

var (
	ErrAlreadyRunning = errors.New("app: capture already running")
	ErrNotRunning     = errors.New("app: capture not running")
	ErrNoFrame        = errors.New("app: no frame available")
)

const (
	previewInterval = 66 * time.Millisecond // ~15 fps
)

type Service struct {
	cfgManager *config.Manager

	mu     sync.RWMutex
	params config.AllParams

	camera     *camera.Manager
	tracker    *tracking.Tracker
	controller mouse.Controller
	dwell      *mouse.DwellState

	preview *stream.PreviewEncoder
	broker  *stream.Broker

	ctx     context.Context
	cancel  context.CancelFunc
	running bool

	trackingEnabled bool

	lastFrame gocv.Mat
}

func NewService(cfg *config.Manager) (*Service, error) {
	params, err := cfg.Load()
	if err != nil {
		return nil, err
	}

	trackerParams := tracking.Params{
		TemplateSize:     params.Tracking.TemplateSizePx,
		SearchMargin:     params.Tracking.SearchMarginPx,
		ScoreThreshold:   float32(params.Tracking.ScoreThreshold),
		AdaptiveTemplate: params.Tracking.AdaptiveTemplate,
		TemplateAlpha:    float32(params.Tracking.TemplateUpdateAlpha),
	}

	tracker := tracking.NewTracker(trackerParams)
	cam := camera.NewManager(0)
	controller := mouse.NewRobotController()
	dwell := mouse.NewDwellState(controller, mouse.DwellParams{
		Enabled:     params.Clicking.DwellEnabled,
		DwellTime:   time.Duration(params.Clicking.DwellTimeMs) * time.Millisecond,
		RadiusPx:    float64(params.Clicking.DwellRadiusPx),
		ClickButton: mapClick(params.Clicking.ClickType, params.Clicking.RightClickToggle),
	})

	return &Service{
		cfgManager:      cfg,
		params:          params,
		camera:          cam,
		tracker:         tracker,
		controller:      controller,
		dwell:           dwell,
		preview:         stream.NewPreviewEncoder(previewInterval),
		broker:          stream.NewBroker(),
		trackingEnabled: true,
		lastFrame:       gocv.NewMat(),
	}, nil
}

func (s *Service) Broker() *stream.Broker {
	return s.broker
}

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
	s.mu.Lock()
	s.lastFrame.Close()
	s.lastFrame = gocv.NewMat()
	s.mu.Unlock()

	return nil
}

func (s *Service) handleFrame(frame camera.Frame) {
	defer frame.Mat.Close()

	s.mu.Lock()
	if !s.lastFrame.Empty() {
		s.lastFrame.Close()
	}
	s.lastFrame = frame.Mat.Clone()
	s.mu.Unlock()

	var result tracking.Result
	var score float32
	var lost bool

	if s.trackingEnabled {
		trackingFrame := tracking.Frame{Mat: frame.Mat, Timestamp: frame.Timestamp}
		if res, err := s.tracker.Update(trackingFrame); err == nil {
			result = res
			score = res.Score
			lost = false
		} else if errors.Is(err, tracking.ErrNoTemplate) {
			lost = true
		} else {
			lost = true
		}
	} else {
		lost = true
	}

	markerColor := color.RGBA{0, 255, 0, 0}
	if lost {
		markerColor = color.RGBA{255, 0, 0, 0}
	}

	overlay.Draw(&frame.Mat, overlay.Marker{
		Point: result.Point,
		Shape: string(s.params.Tracking.MarkerShape),
		Color: markerColor,
		Size:  s.params.Tracking.TemplateSizePx,
		Lost:  lost,
		Score: score,
	})

	if preview, ok := s.preview.Encode(frame.Mat); ok {
		s.broker.EmitPreview(preview)
	}

	telemetry := stream.Telemetry{
		FPS:      frame.FPS,
		Score:    score,
		Lost:     lost,
		Tracking: s.trackingEnabled,
		PosX:     result.Point.X,
		PosY:     result.Point.Y,
	}
	s.broker.EmitTelemetry(telemetry)

	if s.dwell != nil {
		x, y, err := s.controller.CurrentPosition()
		if err == nil {
			s.dwell.Update(x, y, lost)
		}
	}
}

func (s *Service) SetPickPoint(point image.Point) error {
	s.mu.RLock()
	frame := s.lastFrame.Clone()
	s.mu.RUnlock()
	defer frame.Close()

	if frame.Empty() {
		return ErrNoFrame
	}

	return s.tracker.SetPickPoint(tracking.Frame{Mat: frame, Timestamp: time.Now()}, point)
}

func (s *Service) ToggleTracking(enabled bool) {
	s.mu.Lock()
	s.trackingEnabled = enabled
	s.mu.Unlock()
}

func (s *Service) Recenter() error {
	s.mu.RLock()
	frame := s.lastFrame.Clone()
	s.mu.RUnlock()
	defer frame.Close()

	if frame.Empty() {
		return ErrNoFrame
	}

	point := image.Point{X: frame.Cols() / 2, Y: frame.Rows() / 2}
	return s.tracker.SetPickPoint(tracking.Frame{Mat: frame, Timestamp: time.Now()}, point)
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
	s.mu.Unlock()
}

func (s *Service) SaveParams(next config.AllParams) error {
	s.UpdateParams(next)
	return s.cfgManager.Save(next)
}

func (s *Service) applyRuntimeParamsLocked() {
	trackerParams := tracking.Params{
		TemplateSize:     s.params.Tracking.TemplateSizePx,
		SearchMargin:     s.params.Tracking.SearchMarginPx,
		ScoreThreshold:   float32(s.params.Tracking.ScoreThreshold),
		AdaptiveTemplate: s.params.Tracking.AdaptiveTemplate,
		TemplateAlpha:    float32(s.params.Tracking.TemplateUpdateAlpha),
	}
	s.tracker.UpdateParams(trackerParams)

	if s.dwell != nil {
		s.dwell.SetParams(mouse.DwellParams{
			Enabled:     s.params.Clicking.DwellEnabled,
			DwellTime:   time.Duration(s.params.Clicking.DwellTimeMs) * time.Millisecond,
			RadiusPx:    float64(s.params.Clicking.DwellRadiusPx),
			ClickButton: mapClick(s.params.Clicking.ClickType, s.params.Clicking.RightClickToggle),
		})
	}
}

func mapClick(click config.ClickType, rightToggle bool) mouse.ClickButton {
	if rightToggle {
		return mouse.ClickRight
	}

	switch click {
	case config.ClickTypeRight:
		return mouse.ClickRight
	case config.ClickTypeDouble:
		return mouse.ClickLeft // placeholder: double click later
	default:
		return mouse.ClickLeft
	}
}
