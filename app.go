package main

import (
	"context"
	"image"

	appsvc "open-camera-mouse/internal/app"
	"open-camera-mouse/internal/config"
	"open-camera-mouse/internal/stream"

	"github.com/wailsapp/wails/v2/pkg/runtime"
)

// App bridges Wails bindings to the backend service.
type App struct {
	ctx     context.Context
	service *appsvc.Service
}

func NewApp() (*App, error) {
	cfg, err := config.NewManager("open-camera-mouse")
	if err != nil {
		return nil, err
	}

	svc, err := appsvc.NewService(cfg)
	if err != nil {
		return nil, err
	}

	return &App{service: svc}, nil
}

func (a *App) startup(ctx context.Context) {
	a.ctx = ctx

	broker := a.service.Broker()
	broker.SubscribePreview(func(frame stream.PreviewFrame) {
		runtime.EventsEmit(ctx, "preview:frame", frame)
	})
	broker.SubscribeTelemetry(func(t stream.Telemetry) {
		runtime.EventsEmit(ctx, "telemetry:state", t)
	})
}

func (a *App) Start() error {
	return a.service.Start(a.ctx)
}

func (a *App) Stop() error {
	return a.service.Stop()
}

func (a *App) GetParams() config.AllParams {
	return a.service.GetParams()
}

func (a *App) UpdateParams(params config.AllParams) {
	a.service.UpdateParams(params)
}

func (a *App) SaveParams(params config.AllParams) error {
	return a.service.SaveParams(params)
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
