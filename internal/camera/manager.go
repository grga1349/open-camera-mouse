package camera

import (
	"context"
	"errors"
	"sync"
	"time"

	"gocv.io/x/gocv"
)

var (
	ErrAlreadyRunning = errors.New("camera: capture already running")
	ErrHandlerNil     = errors.New("camera: frame handler is nil")
)

type Frame struct {
	Mat       gocv.Mat
	Timestamp time.Time
	FPS       float64
}

type FrameHandler func(Frame)

type Manager struct {
	deviceID int

	mu      sync.RWMutex
	capture *gocv.VideoCapture
	running bool
	fps     float64
	cancel  context.CancelFunc
	wait    sync.WaitGroup
}

func NewManager(deviceID int) *Manager {
	return &Manager{deviceID: deviceID}
}

func (m *Manager) Start(ctx context.Context, handler FrameHandler) error {
	if handler == nil {
		return ErrHandlerNil
	}

	m.mu.Lock()
	if m.running {
		m.mu.Unlock()
		return ErrAlreadyRunning
	}

	cap, err := gocv.VideoCaptureDevice(m.deviceID)
	if err != nil {
		m.mu.Unlock()
		return err
	}

	captureCtx, cancel := context.WithCancel(ctx)
	m.capture = cap
	m.cancel = cancel
	m.running = true
	m.mu.Unlock()

	m.wait.Add(1)
	go m.captureLoop(captureCtx, handler)

	return nil
}

func (m *Manager) Stop() {
	m.mu.Lock()
	if !m.running {
		m.mu.Unlock()
		return
	}

	if m.cancel != nil {
		m.cancel()
	}
	cap := m.capture
	m.capture = nil
	m.running = false
	m.mu.Unlock()

	m.wait.Wait()
	if cap != nil {
		cap.Close()
	}
}

func (m *Manager) captureLoop(ctx context.Context, handler FrameHandler) {
	defer m.wait.Done()

	frame := gocv.NewMat()
	defer frame.Close()

	last := time.Now()
	for {
		select {
		case <-ctx.Done():
			return
		default:
		}

		if ok := m.readFrame(&frame); !ok {
			time.Sleep(10 * time.Millisecond)
			continue
		}

		now := time.Now()
		fps := m.updateFPS(now.Sub(last))
		last = now

		handler(Frame{
			Mat:       frame.Clone(),
			Timestamp: now,
			FPS:       fps,
		})
	}
}

func (m *Manager) readFrame(mat *gocv.Mat) bool {
	m.mu.RLock()
	cap := m.capture
	m.mu.RUnlock()

	if cap == nil {
		return false
	}

	return cap.Read(mat) && !mat.Empty()
}

func (m *Manager) updateFPS(delta time.Duration) float64 {
	fps := 0.0
	if delta > 0 {
		fps = 1.0 / delta.Seconds()
	}

	m.mu.Lock()
	m.fps = fps
	m.mu.Unlock()

	return fps
}

func (m *Manager) FPS() float64 {
	m.mu.RLock()
	defer m.mu.RUnlock()
	return m.fps
}
