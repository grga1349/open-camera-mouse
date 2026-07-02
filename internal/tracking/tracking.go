package tracking

import (
	"errors"
	"image"

	"gocv.io/x/gocv"
)

const (
	searchMarginMultiplier = 2
	scoreThreshold         = 0.68
)

var errInvalidPick = errors.New("tracking: invalid pick point")

type Params struct {
	TemplateSizePx int
}

type Result struct {
	Lost bool
	X    int
	Y    int
}

type Tracker struct {
	params        Params
	template      gocv.Mat
	templatePoint image.Point
	hasTemplate   bool
}

func New(params Params) *Tracker {
	return &Tracker{
		params:   params,
		template: gocv.NewMat(),
	}
}

func (t *Tracker) SetParams(params Params) {
	t.params = params
}

func (t *Tracker) HasTemplate() bool {
	return t.hasTemplate
}

func (t *Tracker) Pick(frame gocv.Mat, x, y int) error {
	size := t.params.TemplateSizePx
	if size <= 0 {
		return errInvalidPick
	}

	gray := toGray(frame)
	defer gray.Close()

	cx := clamp(x-size/2, 0, gray.Cols()-size)
	cy := clamp(y-size/2, 0, gray.Rows()-size)
	rect := image.Rect(cx, cy, cx+size, cy+size)

	roi := gray.Region(rect)
	t.template.Close()
	t.template = roi.Clone()
	roi.Close()

	t.templatePoint = image.Point{X: cx + size/2, Y: cy + size/2}
	t.hasTemplate = true
	return nil
}

func (t *Tracker) Update(frame gocv.Mat) Result {
	fallback := Result{Lost: true, X: t.templatePoint.X, Y: t.templatePoint.Y}

	if !t.hasTemplate || t.template.Empty() {
		return Result{Lost: true}
	}

	gray := toGray(frame)
	defer gray.Close()

	margin := t.params.TemplateSizePx * searchMarginMultiplier
	searchRect := computeSearchRect(gray, t.templatePoint, margin)
	if searchRect.Empty() {
		return fallback
	}

	resultCols := searchRect.Dx() - t.template.Cols() + 1
	resultRows := searchRect.Dy() - t.template.Rows() + 1
	if resultCols <= 0 || resultRows <= 0 {
		return fallback
	}

	searchMat := gray.Region(searchRect)
	defer searchMat.Close()

	response := gocv.NewMatWithSize(resultRows, resultCols, gocv.MatTypeCV32F)
	defer response.Close()
	mask := gocv.NewMat()
	defer mask.Close()

	gocv.MatchTemplate(searchMat, t.template, &response, gocv.TmCcoeffNormed, mask)

	_, maxVal, _, maxLoc := gocv.MinMaxLoc(response)
	if float64(maxVal) < scoreThreshold {
		return fallback
	}

	topLeft := image.Point{
		X: searchRect.Min.X + maxLoc.X,
		Y: searchRect.Min.Y + maxLoc.Y,
	}
	center := image.Point{
		X: topLeft.X + t.template.Cols()/2,
		Y: topLeft.Y + t.template.Rows()/2,
	}
	t.templatePoint = center

	return Result{X: center.X, Y: center.Y}
}

func (t *Tracker) Close() {
	t.template.Close()
}

func toGray(frame gocv.Mat) gocv.Mat {
	gray := gocv.NewMat()
	if frame.Channels() > 1 {
		gocv.CvtColor(frame, &gray, gocv.ColorBGRToGray)
	} else {
		frame.CopyTo(&gray)
	}
	return gray
}

func computeSearchRect(gray gocv.Mat, center image.Point, margin int) image.Rectangle {
	x := clamp(center.X-margin, 0, gray.Cols())
	y := clamp(center.Y-margin, 0, gray.Rows())
	x2 := clamp(center.X+margin, 0, gray.Cols())
	y2 := clamp(center.Y+margin, 0, gray.Rows())
	return image.Rect(x, y, x2, y2)
}

func clamp(v, lo, hi int) int {
	if v < lo {
		return lo
	}
	if v > hi {
		return hi
	}
	return v
}
