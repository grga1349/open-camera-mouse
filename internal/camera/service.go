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
	cap, err := gocv.VideoCaptureDevice(m.deviceID)
	if err != nil {
		return nil, err
	}

	ch := make(chan Frame, 1)
	go func() {
		defer cap.Close()
		defer close(ch)

		frame := gocv.NewMat()
		defer frame.Close()

		last := time.Now()
		for {
			select {
			case <-ctx.Done():
				return
			default:
			}

			if ok := cap.Read(&frame); !ok || frame.Empty() {
				time.Sleep(10 * time.Millisecond)
				continue
			}

			now := time.Now()
			fps := 0.0
			if d := now.Sub(last); d > 0 {
				fps = 1.0 / d.Seconds()
			}
			last = now

			select {
			case ch <- Frame{Mat: frame.Clone(), Timestamp: now, FPS: fps}:
			case <-ctx.Done():
				return
			}
		}
	}()

	return ch, nil
}
