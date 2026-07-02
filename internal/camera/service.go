package camera

import (
	"context"
	"time"

	"gocv.io/x/gocv"
)

type Frame struct {
	Mat       gocv.Mat
	Timestamp time.Time
	FPS       float64
}

type Service struct {
	deviceID int
}

func NewService(deviceID int) *Service {
	return &Service{deviceID: deviceID}
}

func (m *Service) Stream(ctx context.Context) (<-chan Frame, error) {
	vcap, err := gocv.VideoCaptureDevice(m.deviceID)
	if err != nil {
		return nil, err
	}

	ch := make(chan Frame, 1)
	go func() {
		defer vcap.Close()
		defer close(ch)

		frame := gocv.NewMat()
		defer frame.Close()

		const alpha = 0.1
		last := time.Now()
		smoothedFPS := 0.0
		for {
			select {
			case <-ctx.Done():
				return
			default:
			}

			if ok := vcap.Read(&frame); !ok || frame.Empty() {
				time.Sleep(10 * time.Millisecond)
				continue
			}

			now := time.Now()
			if d := now.Sub(last); d > 0 {
				instant := 1.0 / d.Seconds()
				if smoothedFPS == 0 {
					smoothedFPS = instant
				} else {
					smoothedFPS = alpha*instant + (1-alpha)*smoothedFPS
				}
			}
			last = now

			select {
			case ch <- Frame{Mat: frame.Clone(), Timestamp: now, FPS: smoothedFPS}:
			case <-ctx.Done():
				return
			}
		}
	}()

	return ch, nil
}
