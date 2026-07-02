package main

import (
	"context"
	"errors"
	"log"

	appsvc "open-camera-mouse/internal/app"
	"open-camera-mouse/internal/config"
	"open-camera-mouse/internal/hotkeys"
	"open-camera-mouse/internal/preview"

	"github.com/wailsapp/wails/v2/pkg/runtime"
)

type App struct {
	ctx context.Context
	app *appsvc.App
}

func NewApp() (*App, error) {
	cfg, err := config.NewManager("open-camera-mouse")
	if err != nil {
		return nil, err
	}

	inner, err := appsvc.NewApp(cfg)
	if err != nil {
		return nil, err
	}

	if hk, err := hotkeys.NewService(); err == nil {
		inner.Hotkeys = hk
	} else if errors.Is(err, hotkeys.ErrUnsupported) {
		log.Printf("global hotkeys not supported on this platform")
	} else {
		log.Printf("hotkeys unavailable: %v", err)
	}

	return &App{app: inner}, nil
}

func (a *App) startup(ctx context.Context) {
	a.ctx = ctx

	a.app.EmitPreview = func(f preview.Frame) {
		runtime.EventsEmit(ctx, "preview:frame", f)
	}
	a.app.EmitStatus = func(s appsvc.Status) {
		runtime.EventsEmit(ctx, "status:update", s)
	}
	a.app.EmitRunning = func(running bool) {
		runtime.EventsEmit(ctx, "service:running", running)
	}

	params := a.app.GetParams()
	a.applyHotkeys(params)

	if params.AutoStart {
		go func() {
			if err := a.Start(); err != nil {
				a.logErrorf("autostart failed: %v", err)
			}
		}()
	}
}

func (a *App) Start() error {
	if err := a.app.Start(a.ctx); err != nil {
		return err
	}
	runtime.EventsEmit(a.ctx, "service:running", true)
	return nil
}

func (a *App) Stop() error {
	if err := a.app.Stop(); err != nil {
		return err
	}
	runtime.EventsEmit(a.ctx, "service:running", false)
	return nil
}

func (a *App) PickPoint(x, y int) {
	a.app.SendPickPoint(x, y)
}

func (a *App) Recenter() {
	a.app.SendRecenter()
}

func (a *App) ResetMouse() {
	a.app.SendResetMouse()
}

func (a *App) ToggleTracking(enabled bool) {
	a.app.SendSetTrackingEnabled(enabled)
}

func (a *App) GetParams() config.Params {
	return a.app.GetParams()
}

func (a *App) UpdateParams(params config.Params) error {
	if err := a.app.UpdateParams(params); err != nil {
		return err
	}
	a.applyHotkeys(params)
	return nil
}

func (a *App) applyHotkeys(params config.Params) {
	if a.app.Hotkeys == nil {
		return
	}
	actions := map[string]hotkeys.Action{}
	if params.StartPause != "" {
		actions[params.StartPause] = a.toggleStartStop
	}
	if params.Recenter != "" {
		actions[params.Recenter] = func() {
			a.app.SendRecenter()
		}
	}
	if err := a.app.Hotkeys.Update(actions); err != nil {
		a.logErrorf("hotkey update failed: %v", err)
	}
}

func (a *App) toggleStartStop() {
	if a.app.IsRunning() {
		if err := a.Stop(); err != nil {
			a.logErrorf("stop failed: %v", err)
		}
		return
	}
	if err := a.Start(); err != nil {
		a.logErrorf("start failed: %v", err)
	}
}

func (a *App) shutdown(ctx context.Context) {
	if a.app.IsRunning() {
		_ = a.app.Stop()
	}
	if a.app.Hotkeys != nil {
		a.app.Hotkeys.Close()
	}
}

func (a *App) logErrorf(format string, args ...interface{}) {
	if a.ctx != nil {
		runtime.LogErrorf(a.ctx, format, args...)
		return
	}
	log.Printf(format, args...)
}
