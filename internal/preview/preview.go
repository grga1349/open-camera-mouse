package preview

import (
	"encoding/base64"
	"time"

	"gocv.io/x/gocv"
	"open-camera-mouse/internal/camera"
)

const (
	previewInterval = 66 * time.Millisecond
	jpegQuality     = 80
)

type TrackingOverlay struct {
	X              int  `json:"x"`
	Y              int  `json:"y"`
	TemplateSizePx int  `json:"templateSizePx"`
	Lost           bool `json:"lost"`
}

type Frame struct {
	DataURL  string           `json:"dataUrl"`
	Width    int              `json:"width"`
	Height   int              `json:"height"`
	Tracking *TrackingOverlay `json:"tracking,omitempty"`
}

type Encoder struct {
	lastEncode time.Time
}

func NewEncoder() *Encoder {
	return &Encoder{}
}

func (e *Encoder) Encode(frame camera.Frame, tracking *TrackingOverlay) *Frame {
	if time.Since(e.lastEncode) < previewInterval {
		return nil
	}
	e.lastEncode = time.Now()

	display := gocv.NewMat()
	defer display.Close()
	gocv.Flip(frame.Mat, &display, 1)

	buf, err := gocv.IMEncodeWithParams(gocv.JPEGFileExt, display, []int{gocv.IMWriteJpegQuality, jpegQuality})
	if err != nil {
		return nil
	}
	defer buf.Close()

	dataURL := "data:image/jpeg;base64," + base64.StdEncoding.EncodeToString(buf.GetBytes())

	return &Frame{
		DataURL:  dataURL,
		Width:    display.Cols(),
		Height:   display.Rows(),
		Tracking: tracking,
	}
}
