package camera

import (
	"context"
	"time"

	"gocv.io/x/gocv"
)

type Frame struct {
	Mat    gocv.Mat
	Width  int
	Height int
}

type Service struct {
	deviceID int
}

func NewService(deviceID int) *Service {
	return &Service{deviceID: deviceID}
}

func (s *Service) Stream(ctx context.Context) (<-chan Frame, error) {
	vcap, err := gocv.VideoCaptureDevice(s.deviceID)
	if err != nil {
		return nil, err
	}

	ch := make(chan Frame, 1)
	go func() {
		defer vcap.Close()
		defer close(ch)

		frame := gocv.NewMat()
		defer frame.Close()

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

			select {
			case ch <- Frame{
				Mat:    frame.Clone(),
				Width:  frame.Cols(),
				Height: frame.Rows(),
			}:
			case <-ctx.Done():
				return
			}
		}
	}()

	return ch, nil
}
