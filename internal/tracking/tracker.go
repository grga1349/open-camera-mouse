package tracking

import (
	"errors"
	"image"
	"sync"
	"time"

	"open-camera-mouse/internal/camera"

	"gocv.io/x/gocv"
)

var (
	ErrNoTemplate     = errors.New("tracking: template not set")
	ErrInvalidPick    = errors.New("tracking: invalid pick point")
	ErrNoSearchRegion = errors.New("tracking: search region empty")
	ErrNoFrame        = errors.New("tracking: no frame available")
)

type Params struct {
	TemplateSize     int
	SearchMargin     int
	ScoreThreshold   float32
	AdaptiveTemplate bool
	TemplateAlpha    float32
	MarkerShape      string
}

type Result struct {
	Point     image.Point
	Score     float32
	Lost      bool
	Timestamp time.Time
}

type Tracker struct {
	mu              sync.RWMutex
	params          Params
	template        gocv.Mat
	templatePoint   image.Point
	lastFrame       gocv.Mat
	trackingEnabled bool
	lost            bool
}

func NewTracker(params Params) *Tracker {
	return &Tracker{
		params:          params,
		template:        gocv.NewMat(),
		lastFrame:       gocv.NewMat(),
		trackingEnabled: true,
	}
}

func (t *Tracker) UpdateParams(params Params) {
	t.mu.Lock()
	defer t.mu.Unlock()
	t.params = params
}

func (t *Tracker) Snapshot() Params {
	t.mu.RLock()
	defer t.mu.RUnlock()
	return t.params
}

func (t *Tracker) SetTrackingEnabled(enabled bool) {
	t.mu.Lock()
	defer t.mu.Unlock()
	t.trackingEnabled = enabled
}

func (t *Tracker) IsTrackingEnabled() bool {
	t.mu.RLock()
	defer t.mu.RUnlock()
	return t.trackingEnabled
}

func (t *Tracker) Update(frame camera.Frame) Result {
	t.mu.Lock()
	defer t.mu.Unlock()

	if !t.lastFrame.Empty() {
		t.lastFrame.Close()
	}
	t.lastFrame = frame.Mat.Clone()

	base := Result{Lost: true, Point: t.templatePoint, Timestamp: frame.Timestamp}

	if !t.trackingEnabled || t.template.Empty() {
		return base
	}

	gray := t.toGray(frame)
	defer gray.Close()

	searchRect, err := t.computeSearchRect(gray)
	if err != nil {
		t.lost = true
		return base
	}

	maxVal, maxLoc, err := t.matchTemplate(gray, searchRect)
	if err != nil {
		t.lost = true
		return base
	}

	result := t.buildResult(searchRect, maxLoc, maxVal, frame.Timestamp)

	if t.params.AdaptiveTemplate {
		t.applyAdaptiveTemplate(gray, searchRect, maxLoc)
	}

	t.templatePoint = result.Point
	t.lost = false

	return result
}

func (t *Tracker) SetPickPoint(displayPoint image.Point) error {
	t.mu.Lock()
	defer t.mu.Unlock()

	if t.lastFrame.Empty() {
		return ErrNoFrame
	}

	point := displayPoint
	if t.lastFrame.Cols() > 0 {
		point.X = t.lastFrame.Cols() - displayPoint.X
	}

	return t.extractTemplate(t.lastFrame, point)
}

func (t *Tracker) Recenter() error {
	t.mu.Lock()
	defer t.mu.Unlock()

	if t.lastFrame.Empty() {
		return ErrNoFrame
	}

	point := image.Point{X: t.lastFrame.Cols() / 2, Y: t.lastFrame.Rows() / 2}
	return t.extractTemplate(t.lastFrame, point)
}

func (t *Tracker) Close() {
	t.mu.Lock()
	defer t.mu.Unlock()
	t.template.Close()
	t.template = gocv.NewMat()
	if !t.lastFrame.Empty() {
		t.lastFrame.Close()
	}
	t.lastFrame = gocv.NewMat()
}

func (t *Tracker) extractTemplate(frame gocv.Mat, point image.Point) error {
	gray := gocv.NewMat()
	if frame.Channels() > 1 {
		gocv.CvtColor(frame, &gray, gocv.ColorBGRToGray)
	} else {
		frame.CopyTo(&gray)
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
	t.lost = false
	return nil
}

func (t *Tracker) toGray(frame camera.Frame) gocv.Mat {
	gray := gocv.NewMat()
	if frame.Mat.Channels() > 1 {
		gocv.CvtColor(frame.Mat, &gray, gocv.ColorBGRToGray)
	} else {
		frame.Mat.CopyTo(&gray)
	}
	return gray
}

func (t *Tracker) computeSearchRect(gray gocv.Mat) (image.Rectangle, error) {
	searchRect := t.searchRect(gray.Cols(), gray.Rows())
	if searchRect.Empty() {
		return image.Rectangle{}, ErrNoSearchRegion
	}
	resultCols := searchRect.Dx() - t.template.Cols() + 1
	resultRows := searchRect.Dy() - t.template.Rows() + 1
	if resultCols <= 0 || resultRows <= 0 {
		return image.Rectangle{}, ErrNoSearchRegion
	}
	return searchRect, nil
}

func (t *Tracker) matchTemplate(gray gocv.Mat, searchRect image.Rectangle) (float64, image.Point, error) {
	searchMat := gray.Region(searchRect)
	defer searchMat.Close()

	resultCols := searchRect.Dx() - t.template.Cols() + 1
	resultRows := searchRect.Dy() - t.template.Rows() + 1

	response := gocv.NewMatWithSize(resultRows, resultCols, gocv.MatTypeCV32F)
	defer response.Close()
	mask := gocv.NewMat()
	defer mask.Close()

	gocv.MatchTemplate(searchMat, t.template, &response, gocv.TmCcoeffNormed, mask)

	_, maxVal, _, maxLoc := gocv.MinMaxLoc(response)
	if float32(maxVal) < t.params.ScoreThreshold {
		return 0, image.Point{}, errors.New("below threshold")
	}
	return float64(maxVal), maxLoc, nil
}

func (t *Tracker) buildResult(searchRect image.Rectangle, maxLoc image.Point, maxVal float64, ts time.Time) Result {
	topLeft := image.Point{X: searchRect.Min.X + maxLoc.X, Y: searchRect.Min.Y + maxLoc.Y}
	center := image.Point{X: topLeft.X + t.template.Cols()/2, Y: topLeft.Y + t.template.Rows()/2}
	return Result{Point: center, Score: float32(maxVal), Lost: false, Timestamp: ts}
}

func (t *Tracker) applyAdaptiveTemplate(gray gocv.Mat, searchRect image.Rectangle, maxLoc image.Point) {
	searchMat := gray.Region(searchRect)
	defer searchMat.Close()
	t.updateTemplate(searchMat, maxLoc)
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
