package tracking

import (
	"context"
	"image"

	"open-camera-mouse/internal/camera"
)

type FrameResult struct {
	Frame   camera.Frame
	Point   image.Point
	Score   float32
	Lost    bool
	Enabled bool
	Params  Params
}

func (t *Service) Stream(ctx context.Context, frames <-chan camera.Frame) <-chan FrameResult {
	out := make(chan FrameResult, 1)
	go func() {
		defer close(out)
		defer t.Close()
		for {
			select {
			case <-ctx.Done():
				drainFrames(frames)
				return
			case frame, ok := <-frames:
				if !ok {
					return
				}
				t.drainControlPlane(frame)
				r := t.updateFrame(frame)
				fr := FrameResult{
					Frame:   frame,
					Point:   r.Point,
					Score:   r.Score,
					Lost:    r.Lost,
					Enabled: t.IsTrackingEnabled(),
					Params:  t.Snapshot(),
				}
				select {
				case out <- fr:
				case <-ctx.Done():
					frame.Mat.Close()
					drainFrames(frames)
					return
				}
			}
		}
	}()
	return out
}

func drainFrames(ch <-chan camera.Frame) {
	for {
		select {
		case f, ok := <-ch:
			if !ok {
				return
			}
			f.Mat.Close()
		default:
			return
		}
	}
}
