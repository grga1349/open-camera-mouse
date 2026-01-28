package mouse

import (
	"math"
	"time"
)

type DwellParams struct {
	Enabled     bool
	DwellTime   time.Duration
	RadiusPx    float64
	ClickButton ClickButton
}

type DwellState struct {
	params     DwellParams
	controller Controller

	refX       int
	refY       int
	refSet     bool
	dwellStart time.Time
}

func NewDwellState(controller Controller, params DwellParams) *DwellState {
	return &DwellState{controller: controller, params: params}
}

func (d *DwellState) SetParams(params DwellParams) {
	d.params = params
	d.reset(d.refX, d.refY)
}

func (d *DwellState) Update(cursorX, cursorY int, trackingLost bool) {
	if !d.params.Enabled || trackingLost {
		d.reset(cursorX, cursorY)
		return
	}

	if !d.refSet {
		d.refX = cursorX
		d.refY = cursorY
		d.refSet = true
		d.dwellStart = time.Now()
		return
	}

	dist := distance(cursorX, cursorY, d.refX, d.refY)
	if dist > d.params.RadiusPx {
		d.refX = cursorX
		d.refY = cursorY
		d.dwellStart = time.Now()
		return
	}

	if time.Since(d.dwellStart) >= d.params.DwellTime {
		_ = d.controller.Click(d.params.ClickButton)
		d.dwellStart = time.Now()
	}
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
