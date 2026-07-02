package cursor

import (
	"context"
	"image"
	"image/color"
	"math"
	"sync"
	"time"

	"open-camera-mouse/internal/mouse"
	"open-camera-mouse/internal/overlay"
	"open-camera-mouse/internal/stream"
	"open-camera-mouse/internal/tracking"

	"gocv.io/x/gocv"
)

const previewInterval = 66 * time.Millisecond

type Service struct {
	controller mouse.Controller
	mapper     *mouse.Mapper
	dwell      *mouse.DwellState

	// pipeline-goroutine-owned — no lock needed
	lastPoint image.Point
	pointSet  bool

	// control-plane → pipeline dirty flags
	mappingMu      sync.Mutex
	pendingMapping mouse.MappingParams
	mappingDirty   bool
	resetPending   bool
}

func NewService(
	controller mouse.Controller,
	mappingParams mouse.MappingParams,
	dwellParams mouse.DwellParams,
	onDwellClick func(),
) *Service {
	svc := &Service{
		controller: controller,
		mapper:     mouse.NewMapper(mappingParams),
	}
	svc.dwell = mouse.NewDwellState(controller, dwellParams, onDwellClick)
	return svc
}

func (m *Service) Run(ctx context.Context, results <-chan tracking.FrameResult) (<-chan stream.PreviewFrame, <-chan stream.Telemetry, <-chan struct{}) {
	previewCh := make(chan stream.PreviewFrame, 2)
	telemCh := make(chan stream.Telemetry, 4)
	done := make(chan struct{})
	go func() {
		defer close(done)
		defer close(previewCh)
		defer close(telemCh)
		enc := stream.NewPreviewEncoder(previewInterval)
		for {
			select {
			case <-ctx.Done():
				drain(results)
				return
			case result, ok := <-results:
				if !ok {
					return
				}
				m.step(result, enc, previewCh, telemCh)
			}
		}
	}()
	return previewCh, telemCh, done
}

func (m *Service) step(result tracking.FrameResult, enc *stream.PreviewEncoder, previewCh chan<- stream.PreviewFrame, telemCh chan<- stream.Telemetry) {
	defer result.Frame.Mat.Close()
	m.update(result.Point, result.Lost)
	m.updateDwell(result.Lost)
	m.renderAndPublish(result, enc, previewCh, telemCh)
}

func (m *Service) renderAndPublish(result tracking.FrameResult, enc *stream.PreviewEncoder, previewCh chan<- stream.PreviewFrame, telemCh chan<- stream.Telemetry) {
	markerColor := color.RGBA{G: 255}
	if !result.Enabled {
		markerColor = color.RGBA{R: 255, G: 255, B: 255}
	} else if result.Lost {
		markerColor = color.RGBA{R: 255}
	}

	display := result.Frame.Mat.Clone()
	defer display.Close()
	gocv.Flip(display, &display, 1)

	mirroredPoint := result.Point
	if display.Cols() > 0 {
		mirroredPoint = image.Point{X: display.Cols() - result.Point.X, Y: result.Point.Y}
	}

	overlay.Draw(&display, overlay.Marker{
		Point: mirroredPoint,
		Shape: result.Params.MarkerShape,
		Color: markerColor,
		Size:  result.Params.TemplateSize,
		Lost:  result.Lost,
		Score: result.Score,
	})

	if preview, ok := enc.Encode(display); ok {
		select {
		case previewCh <- preview:
		default:
		}
	}

	select {
	case telemCh <- stream.Telemetry{
		FPS:      result.Frame.FPS,
		Score:    result.Score,
		Lost:     result.Lost,
		Tracking: result.Enabled,
		PosX:     result.Point.X,
		PosY:     result.Point.Y,
	}:
	default:
	}
}

func (m *Service) update(point image.Point, lost bool) bool {
	m.mappingMu.Lock()
	pending, dirty, reset := m.pendingMapping, m.mappingDirty, m.resetPending
	m.mappingDirty, m.resetPending = false, false
	m.mappingMu.Unlock()

	if reset {
		m.mapper.Reset()
		m.pointSet = false
	}
	if dirty {
		m.mapper.SetParams(pending)
	}

	if lost {
		m.mapper.Reset()
		m.pointSet = false
		return false
	}

	if !m.pointSet {
		m.mapper.Reset()
		m.pointSet = true
		m.lastPoint = point
		return false
	}

	dx := float64(m.lastPoint.X - point.X)
	dy := float64(point.Y - m.lastPoint.Y)

	moveX, moveY := m.mapper.Update(dx, dy)
	m.lastPoint = point

	if moveX == 0 && moveY == 0 {
		return false
	}

	x, y, err := m.controller.CurrentPosition()
	if err != nil {
		return false
	}

	targetX := int(math.Round(float64(x) + moveX))
	targetY := int(math.Round(float64(y) + moveY))
	_ = m.controller.Move(targetX, targetY)

	return true
}

func (m *Service) updateDwell(lost bool) {
	if m.dwell == nil {
		return
	}

	x, y, err := m.controller.CurrentPosition()
	if err != nil {
		return
	}
	m.dwell.Update(x, y, lost)
}

func (m *Service) Reset() {
	m.mappingMu.Lock()
	m.resetPending = true
	m.mappingMu.Unlock()
}

func (m *Service) SetMappingParams(params mouse.MappingParams) {
	m.mappingMu.Lock()
	m.pendingMapping = params
	m.mappingDirty = true
	m.mappingMu.Unlock()
}

func (m *Service) SetDwellParams(params mouse.DwellParams) {
	if m.dwell != nil {
		m.dwell.SetParams(params)
	}
}

func (m *Service) CenterCursor() {
	if m.controller == nil {
		return
	}
	width, height, err := m.controller.ScreenSize()
	if err != nil {
		return
	}
	_ = m.controller.Move(width/2, height/2)
}

func drain(ch <-chan tracking.FrameResult) {
	for {
		select {
		case r, ok := <-ch:
			if !ok {
				return
			}
			r.Frame.Mat.Close()
		default:
			return
		}
	}
}
