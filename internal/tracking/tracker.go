package tracking

import (
	"errors"
	"image"
	"sync"
	"time"

	"gocv.io/x/gocv"
)

var (
	ErrNoTemplate     = errors.New("tracking: template not set")
	ErrInvalidPick    = errors.New("tracking: invalid pick point")
	ErrNoSearchRegion = errors.New("tracking: search region empty")
)

type Params struct {
	TemplateSize     int
	SearchMargin     int
	ScoreThreshold   float32
	AdaptiveTemplate bool
	TemplateAlpha    float32
}

type Result struct {
	Point     image.Point
	Score     float32
	Timestamp time.Time
}

type Tracker struct {
	mu            sync.RWMutex
	params        Params
	template      gocv.Mat
	templatePoint image.Point
	lastUpdate    time.Time
	lost          bool
}

type Frame struct {
	Mat       gocv.Mat
	Timestamp time.Time
}

func NewTracker(params Params) *Tracker {
	tmpl := gocv.NewMat()
	return &Tracker{params: params, template: tmpl}
}

func (t *Tracker) UpdateParams(params Params) {
	t.mu.Lock()
	defer t.mu.Unlock()
	t.params = params
}

func (t *Tracker) SetPickPoint(frame Frame, point image.Point) error {
	t.mu.Lock()
	defer t.mu.Unlock()

	gray := gocv.NewMat()
	if frame.Mat.Channels() > 1 {
		gocv.CvtColor(frame.Mat, &gray, gocv.ColorBGRToGray)
	} else {
		frame.Mat.CopyTo(&gray)
	}
	defer gray.Close()

	size := t.params.TemplateSize
	if size <= 0 {
		return ErrInvalidPick
	}

	x := clamp(point.X-size/2, 0, gray.Cols()-size)
	y := clamp(point.Y-size/2, 0, gray.Rows()-size)
	rect := image.Rect(x, y, x+size, y+size)

	tmpl := gray.Region(rect)
	t.template.Close()
	t.template = tmpl.Clone()
	tmpl.Close()

	t.templatePoint = image.Point{X: x + size/2, Y: y + size/2}
	t.lastUpdate = frame.Timestamp
	t.lost = false

	return nil
}

func (t *Tracker) Update(frame Frame) (Result, error) {
	t.mu.Lock()
	defer t.mu.Unlock()

	if t.template.Empty() {
		return Result{}, ErrNoTemplate
	}

	gray := gocv.NewMat()
	if frame.Mat.Channels() > 1 {
		gocv.CvtColor(frame.Mat, &gray, gocv.ColorBGRToGray)
	} else {
		frame.Mat.CopyTo(&gray)
	}
	defer gray.Close()

	searchRect := t.searchRect(gray.Cols(), gray.Rows())
	if searchRect.Empty() {
		t.lost = true
		return Result{}, ErrNoSearchRegion
	}

	searchMat := gray.Region(searchRect)
	defer searchMat.Close()

	resultCols := searchRect.Dx() - t.template.Cols() + 1
	resultRows := searchRect.Dy() - t.template.Rows() + 1
	if resultCols <= 0 || resultRows <= 0 {
		t.lost = true
		return Result{}, ErrNoSearchRegion
	}

	response := gocv.NewMatWithSize(resultRows, resultCols, gocv.MatTypeCV32F)
	defer response.Close()
	mask := gocv.NewMat()
	defer mask.Close()

	gocv.MatchTemplate(searchMat, t.template, &response, gocv.TmCcoeffNormed, mask)

	_, maxVal, _, maxLoc := gocv.MinMaxLoc(response)
	if float32(maxVal) < t.params.ScoreThreshold {
		t.lost = true
		return Result{}, nil
	}

	topLeft := image.Point{X: searchRect.Min.X + maxLoc.X, Y: searchRect.Min.Y + maxLoc.Y}
	center := image.Point{X: topLeft.X + t.template.Cols()/2, Y: topLeft.Y + t.template.Rows()/2}

	if t.params.AdaptiveTemplate {
		t.updateTemplate(searchMat, maxLoc)
	}

	t.templatePoint = center
	t.lost = false
	t.lastUpdate = frame.Timestamp

	return Result{
		Point:     center,
		Score:     float32(maxVal),
		Timestamp: frame.Timestamp,
	}, nil
}

func (t *Tracker) updateTemplate(searchMat gocv.Mat, localTopLeft image.Point) {
	alpha := t.params.TemplateAlpha
	if alpha <= 0 {
		return
	}

	roiRect := image.Rect(
		localTopLeft.X,
		localTopLeft.Y,
		localTopLeft.X+t.template.Cols(),
		localTopLeft.Y+t.template.Rows(),
	)
	if roiRect.Max.X > searchMat.Cols() || roiRect.Max.Y > searchMat.Rows() {
		return
	}

	roi := searchMat.Region(roiRect)
	defer roi.Close()

	gocv.AddWeighted(roi, float64(alpha), t.template, float64(1-alpha), 0, &t.template)
}

func (t *Tracker) searchRect(width, height int) image.Rectangle {
	size := t.params.TemplateSize
	margin := t.params.SearchMargin

	x := clamp(t.templatePoint.X-margin, 0, width-size)
	y := clamp(t.templatePoint.Y-margin, 0, height-size)

	x2 := clamp(t.templatePoint.X+margin, size, width)
	y2 := clamp(t.templatePoint.Y+margin, size, height)

	return image.Rect(x, y, x2, y2)
}

func (t *Tracker) Lost() bool {
	t.mu.RLock()
	defer t.mu.RUnlock()
	return t.lost
}

func clamp(val, min, max int) int {
	if val < min {
		return min
	}
	if val > max {
		return max
	}
	return val
}
