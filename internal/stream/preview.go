package stream

import (
	"encoding/base64"
	"time"

	"gocv.io/x/gocv"
)

type PreviewFrame struct {
	Data      string
	Width     int
	Height    int
	Timestamp time.Time
}

type PreviewEncoder struct {
	interval time.Duration
	lastSend time.Time
}

func NewPreviewEncoder(interval time.Duration) *PreviewEncoder {
	return &PreviewEncoder{interval: interval}
}

func (p *PreviewEncoder) Encode(frame gocv.Mat) (PreviewFrame, bool) {
	now := time.Now()
	if p.interval > 0 && now.Sub(p.lastSend) < p.interval {
		return PreviewFrame{}, false
	}

	buf, err := gocv.IMEncode(gocv.JPEGFileExt, frame)
	if err != nil {
		return PreviewFrame{}, false
	}
	defer buf.Close()

	encoded := base64.StdEncoding.EncodeToString(buf.GetBytes())

	p.lastSend = now
	return PreviewFrame{
		Data:      encoded,
		Width:     frame.Cols(),
		Height:    frame.Rows(),
		Timestamp: now,
	}, true
}
