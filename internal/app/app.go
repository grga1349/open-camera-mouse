package app

import (
	"context"
	"errors"
	"sync"

	"open-camera-mouse/internal/camera"
	"open-camera-mouse/internal/config"
	"open-camera-mouse/internal/mouse"
	"open-camera-mouse/internal/preview"
	"open-camera-mouse/internal/tracking"
)

const commandBufferSize = 8

var (
	ErrAlreadyRunning = errors.New("app: already running")
	ErrNotRunning     = errors.New("app: not running")
)

type Status struct {
	Running bool `json:"running"`
	Lost    bool `json:"lost"`
}

type App struct {
	cfg     *config.Manager
	camera  *camera.Service
	tracker *tracking.Tracker
	mouse   *mouse.Mouse

	commands chan command

	EmitPreview func(preview.Frame)
	EmitStatus  func(Status)
	EmitRunning func(bool)

	mu      sync.Mutex
	params  config.Params
	cancel  context.CancelFunc
	done    chan struct{}
	running bool

	// runtime state — only accessed from run goroutine
	trackingEnabled bool
	lastLost        bool
	pendingPick     bool
	pendingPickX    int
	pendingPickY    int
	pendingRecenter bool
	enc             *preview.Encoder
}

func NewApp(cfg *config.Manager) (*App, error) {
	params, err := cfg.Load()
	if err != nil {
		return nil, err
	}

	return &App{
		cfg:      cfg,
		camera:   camera.NewService(0),
		tracker:  tracking.New(tracking.Params{TemplateSizePx: params.TemplateSizePx}),
		mouse:    mouse.New(mouseParams(params)),
		commands: make(chan command, commandBufferSize),
		params:   params,
	}, nil
}

func (a *App) Start(ctx context.Context) error {
	a.mu.Lock()
	if a.running {
		a.mu.Unlock()
		return ErrAlreadyRunning
	}
	runCtx, cancel := context.WithCancel(ctx)
	a.cancel = cancel
	a.done = make(chan struct{})
	a.commands = make(chan command, commandBufferSize)
	a.running = true
	a.mu.Unlock()

	go a.run(runCtx)
	return nil
}

func (a *App) Stop() error {
	a.mu.Lock()
	if !a.running {
		a.mu.Unlock()
		return ErrNotRunning
	}
	cancel := a.cancel
	done := a.done
	a.mu.Unlock()

	cancel()
	<-done
	return nil
}

func (a *App) Close() {
	a.tracker.Close()
}

func (a *App) IsRunning() bool {
	a.mu.Lock()
	defer a.mu.Unlock()
	return a.running
}

func (a *App) GetParams() config.Params {
	a.mu.Lock()
	defer a.mu.Unlock()
	return a.params
}

func (a *App) UpdateParams(p config.Params) error {
	if err := a.cfg.Save(p); err != nil {
		return err
	}
	a.mu.Lock()
	running := a.running
	a.params = p
	a.mu.Unlock()
	if running {
		return a.sendCommand(command{kind: cmdSetParams, params: p})
	}
	return nil
}

func (a *App) SendPickPoint(x, y int) error {
	return a.sendCommand(command{kind: cmdPickPoint, x: x, y: y})
}

func (a *App) SendRecenter() error {
	return a.sendCommand(command{kind: cmdRecenter})
}

func (a *App) SendResetMouse() error {
	return a.sendCommand(command{kind: cmdResetMouse})
}

func (a *App) SendSetTrackingEnabled(enabled bool) error {
	return a.sendCommand(command{kind: cmdSetTrackingEnabled, enabled: enabled})
}

func (a *App) sendCommand(cmd command) error {
	select {
	case a.commands <- cmd:
		return nil
	default:
		return errors.New("app: command queue full")
	}
}

func (a *App) run(ctx context.Context) {
	defer func() {
		a.mu.Lock()
		a.running = false
		close(a.done)
		a.mu.Unlock()
	}()

	frames, err := a.camera.Stream(ctx)
	if err != nil {
		if a.EmitRunning != nil {
			a.EmitRunning(false)
		}
		return
	}

	a.mu.Lock()
	params := a.params
	a.mu.Unlock()
	a.tracker.SetParams(tracking.Params{TemplateSizePx: params.TemplateSizePx})
	a.mouse.SetParams(mouseParams(params))

	a.enc = preview.NewEncoder()
	a.lastLost = true
	a.trackingEnabled = true
	a.mouse.Reset()

	for {
		select {
		case <-ctx.Done():
			return
		case cmd := <-a.commands:
			a.handleCommand(cmd)
		case frame, ok := <-frames:
			if !ok {
				return
			}
			a.handleFrame(frame)
		}
	}
}

func (a *App) handleFrame(frame camera.Frame) {
	defer frame.Mat.Close()

	if a.pendingPick {
		a.pendingPick = false
		x := frame.Width - a.pendingPickX
		_ = a.tracker.Pick(frame.Mat, x, a.pendingPickY)
		a.mouse.Reset()
	}
	if a.pendingRecenter {
		a.pendingRecenter = false
		_ = a.tracker.Pick(frame.Mat, frame.Width/2, frame.Height/2)
		a.mouse.Reset()
	}

	var result tracking.Result
	if a.trackingEnabled {
		result = a.tracker.Update(frame.Mat)
	} else {
		result = tracking.Result{Lost: true}
	}

	a.mouse.Update(result.X, result.Y, result.Lost)

	if result.Lost != a.lastLost {
		a.lastLost = result.Lost
		if a.EmitStatus != nil {
			a.EmitStatus(Status{Running: true, Lost: result.Lost})
		}
	}

	var overlay *preview.TrackingOverlay
	if a.tracker.HasTemplate() {
		overlay = &preview.TrackingOverlay{
			X:              frame.Width - result.X,
			Y:              result.Y,
			TemplateSizePx: a.params.TemplateSizePx,
			Lost:           result.Lost,
		}
	}

	if f := a.enc.Encode(frame, overlay); f != nil && a.EmitPreview != nil {
		a.EmitPreview(*f)
	}
}

func (a *App) handleCommand(cmd command) {
	switch cmd.kind {
	case cmdPickPoint:
		a.pendingPick = true
		a.pendingPickX = cmd.x
		a.pendingPickY = cmd.y
	case cmdRecenter:
		a.pendingRecenter = true
	case cmdSetParams:
		a.tracker.SetParams(tracking.Params{TemplateSizePx: cmd.params.TemplateSizePx})
		a.mouse.SetParams(mouseParams(cmd.params))
	case cmdSetTrackingEnabled:
		a.trackingEnabled = cmd.enabled
		if !cmd.enabled {
			a.mouse.Reset()
		}
	case cmdResetMouse:
		a.mouse.Reset()
	}
}

func mouseParams(p config.Params) mouse.Params {
	return mouse.Params{
		GainMultiplier: p.GainMultiplier,
		Smoothing:      p.Smoothing,
		DwellEnabled:   p.DwellEnabled,
		DwellTimeMs:    p.DwellTimeMs,
	}
}
