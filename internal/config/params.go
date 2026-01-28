package config

type MarkerShape string

const (
	MarkerShapeCircle MarkerShape = "circle"
	MarkerShapeSquare MarkerShape = "square"
)

type ClickType string

const (
	ClickTypeLeft   ClickType = "left"
	ClickTypeRight  ClickType = "right"
	ClickTypeDouble ClickType = "double"
)

type TrackingParams struct {
	TemplateSizePx      int         `json:"templateSizePx"`
	SearchMarginPx      int         `json:"searchMarginPx"`
	ScoreThreshold      float64     `json:"scoreThreshold"`
	AdaptiveTemplate    bool        `json:"adaptiveTemplate"`
	TemplateUpdateAlpha float64     `json:"templateUpdateAlpha"`
	MarkerShape         MarkerShape `json:"markerShape"`
}

type PointerAdvancedParams struct {
	GainX     float64 `json:"gainX"`
	GainY     float64 `json:"gainY"`
	Smoothing float64 `json:"smoothing"`
}

type PointerParams struct {
	Sensitivity int                    `json:"sensitivity"`
	DeadzonePx  int                    `json:"deadzonePx"`
	MaxSpeedPx  int                    `json:"maxSpeedPx"`
	Advanced    *PointerAdvancedParams `json:"advanced"`
}

type ClickingParams struct {
	DwellEnabled     bool      `json:"dwellEnabled"`
	DwellTimeMs      int       `json:"dwellTimeMs"`
	DwellRadiusPx    int       `json:"dwellRadiusPx"`
	ClickType        ClickType `json:"clickType"`
	RightClickToggle bool      `json:"rightClickToggle"`
}

type AllParams struct {
	Tracking TrackingParams `json:"tracking"`
	Pointer  PointerParams  `json:"pointer"`
	Clicking ClickingParams `json:"clicking"`
}

func DefaultParams() AllParams {
	return AllParams{
		Tracking: TrackingParams{
			TemplateSizePx:      30,
			SearchMarginPx:      30,
			ScoreThreshold:      0.60,
			AdaptiveTemplate:    true,
			TemplateUpdateAlpha: 0.20,
			MarkerShape:         MarkerShapeCircle,
		},
		Pointer: PointerParams{
			Sensitivity: 50,
			DeadzonePx:  1,
			MaxSpeedPx:  25,
			Advanced:    nil,
		},
		Clicking: ClickingParams{
			DwellEnabled:     false,
			DwellTimeMs:      500,
			DwellRadiusPx:    30,
			ClickType:        ClickTypeLeft,
			RightClickToggle: false,
		},
	}
}
