package mouse

import (
	"fmt"
	"math"
	"time"

	"github.com/go-vgo/robotgo"
)

const (
	DeadzonePx    = 1.0
	MaxSpeedPx    = 35.0
	DwellRadiusPx = 30.0
)

type ClickButton string

const (
	ClickLeft   ClickButton = "left"
	ClickRight  ClickButton = "right"
	ClickMiddle ClickButton = "middle"
)

type Controller interface {
	Move(x, y int) error
	Click(button ClickButton) error
	CurrentPosition() (int, int, error)
	ScreenSize() (int, int, error)
}

type Params struct {
	GainMultiplier float64
	Smoothing      float64
	DwellEnabled   bool
	DwellTimeMs    int
}

type Mouse struct {
	controller Controller
	params     Params

	lastX   float64
	lastY   float64
	smoothX float64
	smoothY float64
	init    bool

	dwellRefX   int
	dwellRefY   int
	dwellStart  time.Time
	dwellRefSet bool
}

func New(controller Controller, params Params) *Mouse {
	return &Mouse{controller: controller, params: params}
}

func NewRobotController() *robotController {
	return &robotController{}
}

func (m *Mouse) SetParams(params Params) {
	m.params = params
}

func (m *Mouse) Reset() {
	m.init = false
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

	if lost || !m.init {
		if !lost {
			m.init = true
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

	curX, curY, err := m.controller.CurrentPosition()
	if err != nil {
		return
	}

	newX := curX + int(math.Round(m.smoothX))
	newY := curY + int(math.Round(m.smoothY))
	_ = m.controller.Move(newX, newY)
}

func (m *Mouse) updateDwell(lost bool) {
	if !m.params.DwellEnabled || lost {
		m.dwellRefSet = false
		return
	}

	curX, curY, err := m.controller.CurrentPosition()
	if err != nil {
		return
	}

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
		_ = m.controller.Click(ClickLeft)
	}
}

func clampF(v, lo, hi float64) float64 {
	if v < lo {
		return lo
	}
	if v > hi {
		return hi
	}
	return v
}

type robotController struct{}

func (r *robotController) Move(x, y int) error {
	robotgo.Move(x, y)
	return nil
}

func (r *robotController) Click(button ClickButton) error {
	switch button {
	case ClickLeft:
		robotgo.Click("left", false)
	case ClickRight:
		robotgo.Click("right", false)
	case ClickMiddle:
		robotgo.Click("center", false)
	default:
		return fmt.Errorf("mouse: unsupported button %s", button)
	}
	return nil
}

func (r *robotController) CurrentPosition() (int, int, error) {
	x, y := robotgo.Location()
	return x, y, nil
}

func (r *robotController) ScreenSize() (int, int, error) {
	w, h := robotgo.GetScreenSize()
	return w, h, nil
}
