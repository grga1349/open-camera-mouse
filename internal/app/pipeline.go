package app

import (
	"context"
	"image"
	"image/color"
	"time"

	"open-camera-mouse/internal/camera"
	"open-camera-mouse/internal/overlay"
	"open-camera-mouse/internal/stream"
	"open-camera-mouse/internal/tracking"

	"gocv.io/x/gocv"
)

const previewInterval = 66 * time.Millisecond

type FrameResult struct {
	Frame    camera.Frame
	Point    image.Point
	Score    float32
	Lost     bool
	Tracking bool
	Params   tracking.Params
}

func track(ctx context.Context, frames <-chan camera.Frame, t *tracking.Tracker) <-chan FrameResult {
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
				result := t.Update(frame)
				fr := FrameResult{
					Frame:    frame,
					Point:    result.Point,
					Score:    result.Score,
					Lost:     result.Lost,
					Tracking: t.IsTrackingEnabled(),
					Params:   t.Snapshot(),
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

func process(ctx context.Context, results <-chan FrameResult, cursor *CursorMover, broker *stream.Broker) {
	enc := stream.NewPreviewEncoder(previewInterval)
	for {
		select {
		case <-ctx.Done():
			drainResults(results)
			return
		case result, ok := <-results:
			if !ok {
				return
			}
			processFrame(result, cursor, broker, enc)
		}
	}
}

func processFrame(result FrameResult, cursor *CursorMover, broker *stream.Broker, enc *stream.PreviewEncoder) {
	defer result.Frame.Mat.Close()
	cursor.Update(result.Point, result.Lost)
	cursor.UpdateDwell(result.Lost)
	renderAndPublish(result, enc, broker)
}

func renderAndPublish(result FrameResult, enc *stream.PreviewEncoder, broker *stream.Broker) {
	markerColor := color.RGBA{G: 255}
	if !result.Tracking {
		markerColor = color.RGBA{R: 255, G: 255, B: 255}
	} else if result.Lost {
		markerColor = color.RGBA{R: 255}
	}

	display := result.Frame.Mat.Clone()
	defer display.Close()
	gocv.Flip(display, &display, 1)

	mirroredPoint := result.Point
	if display.Cols() > 0 {
		mirroredPoint = image.Point{X: display.Cols() - result.Point.X, Y: result.Point.Y}
	}

	overlay.Draw(&display, overlay.Marker{
		Point: mirroredPoint,
		Shape: result.Params.MarkerShape,
		Color: markerColor,
		Size:  result.Params.TemplateSize,
		Lost:  result.Lost,
		Score: result.Score,
	})

	if preview, ok := enc.Encode(display); ok {
		broker.PublishPreview(preview)
	}

	broker.PublishTelemetry(stream.Telemetry{
		FPS:      result.Frame.FPS,
		Score:    result.Score,
		Lost:     result.Lost,
		Tracking: result.Tracking,
		PosX:     result.Point.X,
		PosY:     result.Point.Y,
	})
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

func drainResults(ch <-chan FrameResult) {
	for {
		select {
		case r, ok := <-ch:
			if !ok {
				return
			}
			r.Frame.Mat.Close()
		default:
			return
		}
	}
}
