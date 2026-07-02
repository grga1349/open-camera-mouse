package tracking

import (
	"errors"
	"image"
	"sync"
	"sync/atomic"

	"open-camera-mouse/internal/camera"

	"gocv.io/x/gocv"
)

var (
	ErrNoTemplate     = errors.New("tracking: template not set")
	ErrInvalidPick    = errors.New("tracking: invalid pick point")
	ErrNoSearchRegion = errors.New("tracking: search region empty")
	ErrNoFrame        = errors.New("tracking: no frame available")

	errBelowThreshold = errors.New("tracking: below threshold")
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
	Point image.Point
	Score float32
	Lost  bool
}

type pendingPick struct {
	point image.Point
	set   bool
}

type Service struct {
	trackingEnabled atomic.Bool

	// control-plane dirty flags — short critical sections, no CV work
	controlMu      sync.Mutex
	pendingParams  Params
	paramsDirty    bool
	pendingPick    pendingPick
	pickDirty      bool
	rencenterDirty bool

	// goroutine-owned state — accessed only from Stream() goroutine
	params        Params
	template      gocv.Mat
	templatePoint image.Point
}

func NewService(params Params) *Service {
	svc := &Service{
		params:   params,
		template: gocv.NewMat(),
	}
	svc.trackingEnabled.Store(true)
	return svc
}

func (t *Service) UpdateParams(params Params) {
	t.controlMu.Lock()
	t.pendingParams = params
	t.paramsDirty = true
	t.controlMu.Unlock()
}

func (t *Service) Snapshot() Params {
	t.controlMu.Lock()
	defer t.controlMu.Unlock()
	if t.paramsDirty {
		return t.pendingParams
	}
	return t.params
}

func (t *Service) SetTrackingEnabled(enabled bool) {
	t.trackingEnabled.Store(enabled)
}

func (t *Service) IsTrackingEnabled() bool {
	return t.trackingEnabled.Load()
}

func (t *Service) SetPickPoint(displayPoint image.Point) error {
	t.controlMu.Lock()
	t.pendingPick = pendingPick{point: displayPoint, set: true}
	t.pickDirty = true
	t.controlMu.Unlock()
	return nil
}

func (t *Service) Recenter() error {
	t.controlMu.Lock()
	t.rencenterDirty = true
	t.controlMu.Unlock()
	return nil
}

func (t *Service) Close() {
	t.controlMu.Lock()
	defer t.controlMu.Unlock()
	t.template.Close()
	t.template = gocv.NewMat()
}

// drainControlPlane applies pending control-plane updates using the incoming frame.
// Must be called only from the Stream() goroutine.
func (t *Service) drainControlPlane(frame camera.Frame) {
	t.controlMu.Lock()
	paramsDirty := t.paramsDirty
	pendingParams := t.pendingParams
	pick := t.pendingPick
	pickDirty := t.pickDirty
	recenter := t.rencenterDirty
	t.paramsDirty = false
	t.pickDirty = false
	t.rencenterDirty = false
	t.controlMu.Unlock()

	if paramsDirty {
		t.params = pendingParams
	}
	if pickDirty && pick.set {
		point := pick.point
		if frame.Mat.Cols() > 0 {
			point.X = frame.Mat.Cols() - pick.point.X
		}
		_ = t.extractTemplateFromFrame(frame.Mat, point)
	}
	if recenter {
		point := image.Point{X: frame.Mat.Cols() / 2, Y: frame.Mat.Rows() / 2}
		_ = t.extractTemplateFromFrame(frame.Mat, point)
	}
}

// updateFrame runs template matching on the given frame without holding any lock.
// Must be called only from the Stream() goroutine.
func (t *Service) updateFrame(frame camera.Frame) Result {
	base := Result{Lost: true, Point: t.templatePoint}

	if !t.trackingEnabled.Load() || t.template.Empty() {
		return base
	}

	gray := t.toGray(frame)
	defer gray.Close()

	searchRect, err := t.computeSearchRect(gray)
	if err != nil {
		return base
	}

	maxVal, maxLoc, err := t.matchTemplate(gray, searchRect)
	if err != nil {
		return base
	}

	result := t.buildResult(searchRect, maxLoc, maxVal)

	if t.params.AdaptiveTemplate {
		t.applyAdaptiveTemplate(gray, searchRect, maxLoc)
	}

	t.templatePoint = result.Point

	return result
}

func (t *Service) extractTemplateFromFrame(frame gocv.Mat, point image.Point) error {
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
	return nil
}

func (t *Service) toGray(frame camera.Frame) gocv.Mat {
	gray := gocv.NewMat()
	if frame.Mat.Channels() > 1 {
		gocv.CvtColor(frame.Mat, &gray, gocv.ColorBGRToGray)
	} else {
		frame.Mat.CopyTo(&gray)
	}
	return gray
}

func (t *Service) computeSearchRect(gray gocv.Mat) (image.Rectangle, error) {
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

func (t *Service) matchTemplate(gray gocv.Mat, searchRect image.Rectangle) (float64, image.Point, error) {
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
		return 0, image.Point{}, errBelowThreshold
	}
	return float64(maxVal), maxLoc, nil
}

func (t *Service) buildResult(searchRect image.Rectangle, maxLoc image.Point, maxVal float64) Result {
	topLeft := image.Point{X: searchRect.Min.X + maxLoc.X, Y: searchRect.Min.Y + maxLoc.Y}
	center := image.Point{X: topLeft.X + t.template.Cols()/2, Y: topLeft.Y + t.template.Rows()/2}
	return Result{Point: center, Score: float32(maxVal), Lost: false}
}

func (t *Service) applyAdaptiveTemplate(gray gocv.Mat, searchRect image.Rectangle, maxLoc image.Point) {
	searchMat := gray.Region(searchRect)
	defer searchMat.Close()
	t.updateTemplate(searchMat, maxLoc)
}

func (t *Service) updateTemplate(searchMat gocv.Mat, localTopLeft image.Point) {
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

func (t *Service) searchRect(width, height int) image.Rectangle {
	size := t.params.TemplateSize
	margin := t.params.SearchMargin

	x := clamp(t.templatePoint.X-margin, 0, width-size)
	y := clamp(t.templatePoint.Y-margin, 0, height-size)

	x2 := clamp(t.templatePoint.X+margin, size, width)
	y2 := clamp(t.templatePoint.Y+margin, size, height)

	return image.Rect(x, y, x2, y2)
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
