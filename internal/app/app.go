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
	recentering     bool
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
	if !a.IsRunning() {
		return ErrNotRunning
	}
	return a.sendCommand(command{kind: cmdPickPoint, x: x, y: y})
}

// SendBeginRecenter pauses tracking/cursor movement and tracking overlay so
// the frontend can guide the user into position before SendConfirmRecenter
// picks the frame center as the new tracking target.
func (a *App) SendBeginRecenter() error {
	if !a.IsRunning() {
		return ErrNotRunning
	}
	return a.sendCommand(command{kind: cmdBeginRecenter})
}

// SendConfirmRecenter picks the current frame's center as the new tracking
// target and resumes tracking. Must follow a SendBeginRecenter call.
func (a *App) SendConfirmRecenter() error {
	if !a.IsRunning() {
		return ErrNotRunning
	}
	return a.sendCommand(command{kind: cmdConfirmRecenter})
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
	a.recentering = false
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
		// pendingPickX/Y arrive in mirrored (display) coordinates — convert
		// to raw-frame space to match frame.Mat, which is never flipped.
		displayX := clampToFrame(a.pendingPickX, frame.Width)
		displayY := clampToFrame(a.pendingPickY, frame.Height)
		rawX := frame.Width - 1 - displayX
		_ = a.tracker.Pick(frame.Mat, rawX, displayY)
		a.mouse.Reset()
	}
	if a.pendingRecenter {
		a.pendingRecenter = false
		a.recentering = false
		_ = a.tracker.Pick(frame.Mat, frame.Width/2, frame.Height/2)
		a.mouse.Reset()
	}

	var result tracking.Result
	switch {
	case a.recentering:
		result = tracking.Result{Lost: true}
	case a.trackingEnabled:
		result = a.tracker.Update(frame.Mat)
	default:
		result = tracking.Result{Lost: true}
	}

	if !a.recentering {
		a.mouse.Update(result.X, result.Y, result.Lost)
	}

	if !a.recentering && result.Lost != a.lastLost {
		a.lastLost = result.Lost
		if a.EmitStatus != nil {
			a.EmitStatus(Status{Running: true, Lost: result.Lost})
		}
	}

	var overlay *preview.TrackingOverlay
	if !a.recentering && a.tracker.HasTemplate() {
		overlay = &preview.TrackingOverlay{
			X:              frame.Width - 1 - result.X,
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
	case cmdBeginRecenter:
		a.recentering = true
	case cmdConfirmRecenter:
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

// clampToFrame keeps a coordinate within the valid pixel index range
// [0, dim-1] for a frame of the given dimension.
func clampToFrame(v, dim int) int {
	if v < 0 {
		return 0
	}
	if v > dim-1 {
		return dim - 1
	}
	return v
}

func mouseParams(p config.Params) mouse.Params {
	return mouse.Params{
		GainMultiplier:    p.GainMultiplier,
		Smoothing:         p.Smoothing,
		DwellEnabled:      p.DwellEnabled,
		DwellTimeMs:       p.DwellTimeMs,
		RightClickEnabled: p.RightClickEnabled,
	}
}
