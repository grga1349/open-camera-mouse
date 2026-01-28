package mouse

import "math"

type MappingParams struct {
	Sensitivity float64
	GainX       float64
	GainY       float64
	Smoothing   float64
	DeadzonePx  float64
	MaxSpeedPx  float64
}

type Mapper struct {
	params MappingParams
	prevX  float64
	prevY  float64
	init   bool
}

func NewMapper(params MappingParams) *Mapper {
	return &Mapper{params: params}
}

func (m *Mapper) Update(dx, dy float64) (float64, float64) {
	p := m.params

	if math.Abs(dx) < p.DeadzonePx {
		dx = 0
	}
	if math.Abs(dy) < p.DeadzonePx {
		dy = 0
	}

	dx = clampFloat(dx, -p.MaxSpeedPx, p.MaxSpeedPx)
	dy = clampFloat(dy, -p.MaxSpeedPx, p.MaxSpeedPx)

	targetX := dx * p.GainX
	targetY := dy * p.GainY

	if !m.init {
		m.prevX = targetX
		m.prevY = targetY
		m.init = true
	}

	smoothedX := lerp(m.prevX, targetX, p.Smoothing)
	smoothedY := lerp(m.prevY, targetY, p.Smoothing)

	m.prevX = smoothedX
	m.prevY = smoothedY

	return smoothedX, smoothedY
}

func clampFloat(value, min, max float64) float64 {
	if value < min {
		return min
	}
	if value > max {
		return max
	}
	return value
}

func lerp(a, b, t float64) float64 {
	return a + (b-a)*t
}
