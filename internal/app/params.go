package app

import (
	"time"

	"open-camera-mouse/internal/config"
	"open-camera-mouse/internal/mouse"
	"open-camera-mouse/internal/tracking"
)

func buildTrackerParams(t config.TrackingParams) tracking.Params {
	return tracking.Params{
		TemplateSize:     t.TemplateSizePx,
		SearchMargin:     t.SearchMarginPx,
		ScoreThreshold:   float32(t.ScoreThreshold),
		AdaptiveTemplate: t.AdaptiveTemplate,
		TemplateAlpha:    float32(t.TemplateUpdateAlpha),
		MarkerShape:      string(t.MarkerShape),
	}
}

func buildMappingParams(p config.PointerParams) mouse.MappingParams {
	gain := mapRange(float64(p.Sensitivity), 1, 100, 1.2, 5.0) * p.Amplification
	smoothing := mapRange(float64(p.Sensitivity), 1, 100, 0.35, 0.15)
	gainX := gain
	gainY := gain

	if adv := p.Advanced; adv != nil {
		if adv.GainX != 0 {
			gainX = adv.GainX
		}
		if adv.GainY != 0 {
			gainY = adv.GainY
		}
		if adv.Smoothing != 0 {
			smoothing = adv.Smoothing
		}
	}

	return mouse.MappingParams{
		Sensitivity: float64(p.Sensitivity),
		GainX:       gainX,
		GainY:       gainY,
		Smoothing:   smoothing,
		DeadzonePx:  max(0, float64(p.DeadzonePx)),
		MaxSpeedPx:  max(1, float64(p.MaxSpeedPx)),
	}
}

func buildDwellParams(c config.ClickingParams) mouse.DwellParams {
	return mouse.DwellParams{
		Enabled:     c.DwellEnabled,
		DwellTime:   time.Duration(c.DwellTimeMs) * time.Millisecond,
		RadiusPx:    float64(c.DwellRadiusPx),
		ClickButton: mapClickButton(c.ClickType, c.RightClickToggle),
	}
}

func mapClickButton(click config.ClickType, rightToggle bool) mouse.ClickButton {
	if rightToggle {
		return mouse.ClickRight
	}
	switch click {
	case config.ClickTypeRight:
		return mouse.ClickRight
	case config.ClickTypeDouble:
		return mouse.ClickLeft
	default:
		return mouse.ClickLeft
	}
}

func mapRange(value, inMin, inMax, outMin, outMax float64) float64 {
	if inMax == inMin {
		return outMin
	}
	if value < inMin {
		value = inMin
	}
	if value > inMax {
		value = inMax
	}
	return (value-inMin)*(outMax-outMin)/(inMax-inMin) + outMin
}
