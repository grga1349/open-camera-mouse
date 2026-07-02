package main

import (
	"context"
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
	hk  *hotkeys.Hotkeys
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

	hk, err := hotkeys.Start(
		a.toggleStartStop,
		func() { _ = a.app.SendRecenter() },
	)
	if err != nil {
		a.logErrorf("hotkeys unavailable: %v", err)
	}
	a.hk = hk

	params := a.app.GetParams()
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

func (a *App) PickPoint(x, y int) error {
	return a.app.SendPickPoint(x, y)
}

func (a *App) Recenter() error {
	return a.app.SendRecenter()
}

func (a *App) ResetMouse() error {
	return a.app.SendResetMouse()
}

func (a *App) ToggleTracking(enabled bool) error {
	return a.app.SendSetTrackingEnabled(enabled)
}

func (a *App) GetParams() config.Params {
	return a.app.GetParams()
}

func (a *App) UpdateParams(params config.Params) error {
	return a.app.UpdateParams(params)
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
	a.hk.Stop()
	a.app.Close()
}

func (a *App) logErrorf(format string, args ...interface{}) {
	if a.ctx != nil {
		runtime.LogErrorf(a.ctx, format, args...)
		return
	}
	log.Printf(format, args...)
}
