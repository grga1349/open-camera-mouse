package mouse

import (
	"math"
	"time"

	"github.com/go-vgo/robotgo"
)

const (
	DeadzonePx    = 1.0
	MaxSpeedPx    = 35.0
	DwellRadiusPx = 30.0
)

type Params struct {
	GainMultiplier float64
	Smoothing      float64
	DwellEnabled   bool
	DwellTimeMs    int
}

type Mouse struct {
	params Params

	lastX       float64
	lastY       float64
	smoothX     float64
	smoothY     float64
	initialized bool

	dwellRefX   int
	dwellRefY   int
	dwellStart  time.Time
	dwellRefSet bool
}

func New(params Params) *Mouse {
	return &Mouse{params: params}
}

func (m *Mouse) SetParams(params Params) {
	m.params = params
}

func (m *Mouse) Reset() {
	m.initialized = false
	m.smoothX = 0
	m.smoothY = 0
	m.dwellRefSet = false
}

func (m *Mouse) Update(x, y int, lost bool) {
	m.updateCursor(x, y, lost)
	m.updateDwell(lost)
}

func (m *Mouse) updateCursor(x, y int, lost bool) {
	fx, fy := float64(x), float64(y)

	if lost || !m.initialized {
		if !lost {
			m.initialized = true
		}
		m.lastX = fx
		m.lastY = fy
		m.smoothX = 0
		m.smoothY = 0
		return
	}

	dx := m.lastX - fx
	dy := fy - m.lastY
	m.lastX = fx
	m.lastY = fy

	if math.Abs(dx) < DeadzonePx {
		dx = 0
	}
	if math.Abs(dy) < DeadzonePx {
		dy = 0
	}

	dx = clampF(dx, -MaxSpeedPx, MaxSpeedPx)
	dy = clampF(dy, -MaxSpeedPx, MaxSpeedPx)

	targetX := dx * m.params.GainMultiplier
	targetY := dy * m.params.GainMultiplier

	m.smoothX += (targetX - m.smoothX) * m.params.Smoothing
	m.smoothY += (targetY - m.smoothY) * m.params.Smoothing

	curX, curY := position()
	newX := curX + int(math.Round(m.smoothX))
	newY := curY + int(math.Round(m.smoothY))
	move(newX, newY)
}

func (m *Mouse) updateDwell(lost bool) {
	if !m.params.DwellEnabled || lost {
		m.dwellRefSet = false
		return
	}

	curX, curY := position()

	if !m.dwellRefSet {
		m.dwellRefX = curX
		m.dwellRefY = curY
		m.dwellRefSet = true
		m.dwellStart = time.Now()
		return
	}

	dist := math.Hypot(float64(curX-m.dwellRefX), float64(curY-m.dwellRefY))
	if dist > DwellRadiusPx {
		m.dwellRefX = curX
		m.dwellRefY = curY
		m.dwellStart = time.Now()
		return
	}

	dwellTime := time.Duration(m.params.DwellTimeMs) * time.Millisecond
	if time.Since(m.dwellStart) >= dwellTime {
		m.dwellStart = time.Now()
		clickLeft()
	}
}

func move(x, y int) { robotgo.Move(x, y) }

func clickLeft() { robotgo.Click("left", false) }

func position() (int, int) { return robotgo.Location() }

func clampF(v, lo, hi float64) float64 {
	if v < lo {
		return lo
	}
	if v > hi {
		return hi
	}
	return v
}
