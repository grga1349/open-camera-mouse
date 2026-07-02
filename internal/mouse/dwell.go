package mouse

import (
	"math"
	"sync"
	"time"
)

type DwellParams struct {
	Enabled     bool
	DwellTime   time.Duration
	RadiusPx    float64
	ClickButton ClickButton
}

type DwellState struct {
	mu         sync.Mutex
	params     DwellParams
	controller Controller
	afterClick func()

	refX       int
	refY       int
	refSet     bool
	dwellStart time.Time
}

func NewDwellState(controller Controller, params DwellParams, afterClick func()) *DwellState {
	return &DwellState{controller: controller, params: params, afterClick: afterClick}
}

func (d *DwellState) SetParams(params DwellParams) {
	d.mu.Lock()
	defer d.mu.Unlock()
	d.params = params
	d.reset(d.refX, d.refY)
}

func (d *DwellState) Update(cursorX, cursorY int, trackingLost bool) {
	d.mu.Lock()

	if !d.params.Enabled || trackingLost {
		d.reset(cursorX, cursorY)
		d.mu.Unlock()
		return
	}

	if !d.refSet {
		d.refX = cursorX
		d.refY = cursorY
		d.refSet = true
		d.dwellStart = time.Now()
		d.mu.Unlock()
		return
	}

	dist := distance(cursorX, cursorY, d.refX, d.refY)
	if dist > d.params.RadiusPx {
		d.refX = cursorX
		d.refY = cursorY
		d.dwellStart = time.Now()
		d.mu.Unlock()
		return
	}

	if time.Since(d.dwellStart) >= d.params.DwellTime {
		btn := d.params.ClickButton
		cb := d.afterClick
		d.dwellStart = time.Now()
		d.mu.Unlock()
		_ = d.controller.Click(btn)
		if cb != nil {
			cb()
		}
		return
	}

	d.mu.Unlock()
}

func (d *DwellState) reset(x, y int) {
	d.refX = x
	d.refY = y
	d.refSet = false
	d.dwellStart = time.Time{}
}

func distance(x1, y1, x2, y2 int) float64 {
	dx := float64(x1 - x2)
	dy := float64(y1 - y2)
	return math.Hypot(dx, dy)
}
