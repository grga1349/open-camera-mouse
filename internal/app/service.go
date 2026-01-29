package app

import (
	"context"
	"errors"
	"image"
	"image/color"
	"math"
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
	cfgManager   *config.Manager
	notifyParams func(config.AllParams)

	mu     sync.RWMutex
	params config.AllParams

	camera      *camera.Manager
	tracker     *tracking.Tracker
	controller  mouse.Controller
	dwell       *mouse.DwellState
	mapper      *mouse.Mapper
	lastPoint   image.Point
	pointSet    bool
	markerPoint image.Point
	markerValid bool

	preview *stream.PreviewEncoder
	broker  *stream.Broker

	ctx     context.Context
	cancel  context.CancelFunc
	running bool

	trackingEnabled bool

	lastFrame gocv.Mat
}

func NewService(cfg *config.Manager, notify func(config.AllParams)) (*Service, error) {
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
	mapper := mouse.NewMapper(pointerMapping(params.Pointer))

	svc := &Service{
		cfgManager:      cfg,
		notifyParams:    notify,
		params:          params,
		camera:          cam,
		tracker:         tracker,
		controller:      controller,
		preview:         stream.NewPreviewEncoder(previewInterval),
		broker:          stream.NewBroker(),
		trackingEnabled: true,
		lastFrame:       gocv.NewMat(),
		mapper:          mapper,
	}

	svc.dwell = mouse.NewDwellState(controller, mouse.DwellParams{
		Enabled:     params.Clicking.DwellEnabled,
		DwellTime:   time.Duration(params.Clicking.DwellTimeMs) * time.Millisecond,
		RadiusPx:    float64(params.Clicking.DwellRadiusPx),
		ClickButton: mapClick(params.Clicking.ClickType, params.Clicking.RightClickToggle),
	}, svc.handleDwellClick)

	return svc, nil
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

func (s *Service) handleDwellClick() {
	s.mu.Lock()
	defer s.mu.Unlock()
	if s.params.Clicking.RightClickToggle {
		s.params.Clicking.RightClickToggle = false
		s.applyRuntimeParamsLocked()
		s.emitParamsLocked()
	}
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

func (s *Service) IsRunning() bool {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.running
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

	s.mu.RLock()
	trackingEnabled := s.trackingEnabled
	savedMarker := s.markerPoint
	hasMarker := s.markerValid
	s.mu.RUnlock()

	if trackingEnabled {
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

	if !lost {
		s.mu.Lock()
		s.markerPoint = result.Point
		s.markerValid = true
		s.mu.Unlock()
	} else if hasMarker {
		result.Point = savedMarker
	}

	if s.mapper != nil {
		if lost {
			s.mapper.Reset()
			s.pointSet = false
		} else {
			if s.pointSet {
				dx := float64(s.lastPoint.X - result.Point.X)
				dy := float64(result.Point.Y - s.lastPoint.Y)
				moveX, moveY := s.mapper.Update(dx, dy)
				if moveX != 0 || moveY != 0 {
					if x, y, err := s.controller.CurrentPosition(); err == nil {
						targetX := int(math.Round(float64(x) + moveX))
						targetY := int(math.Round(float64(y) + moveY))
						_ = s.controller.Move(targetX, targetY)
					}
				}
			} else {
				s.mapper.Reset()
				s.pointSet = true
			}
			if !lost {
				s.lastPoint = result.Point
			}
		}
	}

	markerColor := color.RGBA{0, 255, 0, 0}
	if !trackingEnabled {
		markerColor = color.RGBA{255, 255, 255, 0}
	} else if lost {
		markerColor = color.RGBA{255, 0, 0, 0}
	}

	display := frame.Mat.Clone()
	gocv.Flip(display, &display, 1)
	mirroredPoint := result.Point
	if display.Cols() > 0 {
		mirroredPoint = image.Point{X: display.Cols() - result.Point.X, Y: result.Point.Y}
	}

	overlay.Draw(&display, overlay.Marker{
		Point: mirroredPoint,
		Shape: string(s.params.Tracking.MarkerShape),
		Color: markerColor,
		Size:  s.params.Tracking.TemplateSizePx,
		Lost:  lost,
		Score: score,
	})

	if preview, ok := s.preview.Encode(display); ok {
		s.broker.EmitPreview(preview)
	}
	display.Close()

	telemetry := stream.Telemetry{
		FPS:      frame.FPS,
		Score:    score,
		Lost:     lost,
		Tracking: trackingEnabled,
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

	if frame.Cols() > 0 {
		point.X = frame.Cols() - point.X
	}

	err := s.tracker.SetPickPoint(tracking.Frame{Mat: frame, Timestamp: time.Now()}, point)
	if err == nil {
		s.mu.Lock()
		s.pointSet = false
		s.mapper.Reset()
		s.markerPoint = point
		s.markerValid = true
		s.mu.Unlock()
	}

	return err
}

func (s *Service) ToggleTracking(enabled bool) {
	s.mu.Lock()
	s.trackingEnabled = enabled
	if !enabled {
		s.mapper.Reset()
		s.pointSet = false
	}
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
	err := s.tracker.SetPickPoint(tracking.Frame{Mat: frame, Timestamp: time.Now()}, point)
	if err == nil {
		s.mu.Lock()
		s.pointSet = false
		s.mapper.Reset()
		s.markerPoint = point
		s.markerValid = true
		s.mu.Unlock()
		s.centerCursor()
	}

	return err
}

func (s *Service) centerCursor() {
	if s.controller == nil {
		return
	}
	if width, height, err := s.controller.ScreenSize(); err == nil {
		_ = s.controller.Move(width/2, height/2)
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
	if s.notifyParams == nil {
		return
	}
	params := s.params
	go s.notifyParams(params)
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
	if s.mapper != nil {
		s.mapper.SetParams(pointerMapping(s.params.Pointer))
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

func pointerMapping(p config.PointerParams) mouse.MappingParams {
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
		DeadzonePx:  math.Max(0, float64(p.DeadzonePx)),
		MaxSpeedPx:  math.Max(1, float64(p.MaxSpeedPx)),
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
