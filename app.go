package main

import (
	"context"
	"errors"
	"image"
	"log"

	appsvc "open-camera-mouse/internal/app"
	"open-camera-mouse/internal/config"
	"open-camera-mouse/internal/hotkeys"

	"github.com/wailsapp/wails/v2/pkg/runtime"
)

type App struct {
	ctx     context.Context
	service *appsvc.Service
	hotkeys hotkeys.Service
}

func NewApp() (*App, error) {
	cfg, err := config.NewManager("open-camera-mouse")
	if err != nil {
		return nil, err
	}

	app := &App{}
	svc, err := appsvc.NewService(cfg, app.emitParams)
	if err != nil {
		return nil, err
	}
	app.service = svc
	if hk, err := hotkeys.NewService(); err == nil {
		app.hotkeys = hk
	} else if errors.Is(err, hotkeys.ErrUnsupported) {
		log.Printf("global hotkeys not supported on this platform")
	} else {
		log.Printf("hotkeys unavailable: %v", err)
	}

	return app, nil
}

func (a *App) startup(ctx context.Context) {
	a.ctx = ctx
	params := a.service.GetParams()
	a.applyHotkeys(params.Hotkeys)
	if params.General.AutoStart {
		go func() {
			if err := a.Start(); err != nil {
				a.logErrorf("autostart failed: %v", err)
			}
		}()
	}
}

func (a *App) Start() error {
	previewCh, telemCh, err := a.service.Start(a.ctx)
	if err != nil {
		return err
	}
	go func() {
		for frame := range previewCh {
			runtime.EventsEmit(a.ctx, "preview:frame", frame)
		}
	}()
	go func() {
		for t := range telemCh {
			runtime.EventsEmit(a.ctx, "telemetry:state", t)
		}
	}()
	a.emitRunning(true)
	return nil
}

func (a *App) Stop() error {
	if err := a.service.Stop(); err != nil {
		return err
	}
	a.emitRunning(false)
	return nil
}

func (a *App) GetParams() config.AllParams {
	return a.service.GetParams()
}

func (a *App) UpdateParams(params config.AllParams) {
	a.service.UpdateParams(params)
	a.applyHotkeys(params.Hotkeys)
}

func (a *App) SaveParams(params config.AllParams) error {
	if err := a.service.SaveParams(params); err != nil {
		return err
	}
	a.applyHotkeys(params.Hotkeys)
	return nil
}

func (a *App) SetPickPoint(x, y int) error {
	return a.service.SetPickPoint(image.Pt(x, y))
}

func (a *App) Recenter() error {
	return a.service.Recenter()
}

func (a *App) ToggleTracking(enabled bool) {
	a.service.ToggleTracking(enabled)
}

func (a *App) emitParams(params config.AllParams) {
	if a.ctx == nil {
		return
	}
	runtime.EventsEmit(a.ctx, "params:update", params)
}

func (a *App) emitRunning(running bool) {
	if a.ctx == nil {
		return
	}
	runtime.EventsEmit(a.ctx, "service:running", running)
}

func (a *App) applyHotkeys(binding config.HotkeysParams) {
	if a.hotkeys == nil {
		return
	}
	actions := map[string]hotkeys.Action{}
	if binding.StartPause != "" {
		actions[binding.StartPause] = a.toggleStartStop
	}
	if binding.Recenter != "" {
		actions[binding.Recenter] = func() {
			if a.ctx != nil {
				runtime.EventsEmit(a.ctx, "recenter:hotkey")
			}
		}
	}
	if err := a.hotkeys.Update(actions); err != nil {
		a.logErrorf("hotkey update failed: %v", err)
	}
}

func (a *App) toggleStartStop() {
	if a.service.IsRunning() {
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
	if a.service.IsRunning() {
		_ = a.service.Stop()
	}
	if a.hotkeys != nil {
		a.hotkeys.Close()
	}
}

func (a *App) logErrorf(format string, args ...interface{}) {
	if a.ctx != nil {
		runtime.LogErrorf(a.ctx, format, args...)
		return
	}
	log.Printf(format, args...)
}
