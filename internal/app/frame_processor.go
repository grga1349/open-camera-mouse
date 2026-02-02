package app

import (
	"image"
	"image/color"
	"sync"
	"time"

	"open-camera-mouse/internal/camera"
	"open-camera-mouse/internal/overlay"
	"open-camera-mouse/internal/stream"
	"open-camera-mouse/internal/tracking"

	"gocv.io/x/gocv"
)

// TrackingResult represents the output of frame processing.
type TrackingResult struct {
	Point     image.Point
	Score     float32
	Lost      bool
	Timestamp time.Time
}

// FrameProcessorParams holds configurable parameters for the frame processor.
type FrameProcessorParams struct {
	MarkerShape  string
	TemplateSize int
}

// FrameProcessor handles the frame processing pipeline:
// tracking -> overlay rendering -> preview encoding -> telemetry.
type FrameProcessor struct {
	mu      sync.RWMutex
	tracker *tracking.Tracker
	preview *stream.PreviewEncoder
	broker  *stream.Broker
	params  FrameProcessorParams

	// Last known marker position for display when tracking is lost
	markerPoint image.Point
	markerValid bool

	// Last frame for pick point selection
	lastFrame gocv.Mat

	trackingEnabled bool
}

// NewFrameProcessor creates a new frame processor.
func NewFrameProcessor(
	trackerParams tracking.Params,
	processorParams FrameProcessorParams,
	previewInterval time.Duration,
	broker *stream.Broker,
) *FrameProcessor {
	return &FrameProcessor{
		tracker:         tracking.NewTracker(trackerParams),
		preview:         stream.NewPreviewEncoder(previewInterval),
		broker:          broker,
		params:          processorParams,
		lastFrame:       gocv.NewMat(),
		trackingEnabled: true,
	}
}

// Process handles a single camera frame and returns the tracking result.
func (fp *FrameProcessor) Process(frame camera.Frame) TrackingResult {
	defer frame.Mat.Close()

	// Store frame for pick point
	fp.mu.Lock()
	if !fp.lastFrame.Empty() {
		fp.lastFrame.Close()
	}
	fp.lastFrame = frame.Mat.Clone()
	trackingEnabled := fp.trackingEnabled
	savedMarker := fp.markerPoint
	hasMarker := fp.markerValid
	params := fp.params
	fp.mu.Unlock()

	// Run tracking
	var result TrackingResult
	result.Timestamp = frame.Timestamp
	result.Lost = true

	if trackingEnabled {
		trackingFrame := tracking.Frame{Mat: frame.Mat, Timestamp: frame.Timestamp}
		if res, err := fp.tracker.Update(trackingFrame); err == nil {
			result.Point = res.Point
			result.Score = res.Score
			result.Lost = false
		}
	}

	// Update marker position
	if !result.Lost {
		fp.mu.Lock()
		fp.markerPoint = result.Point
		fp.markerValid = true
		fp.mu.Unlock()
	} else if hasMarker {
		result.Point = savedMarker
	}

	// Render overlay and encode preview
	fp.renderAndEmit(frame, result, trackingEnabled, params)

	// Emit telemetry
	fp.broker.EmitTelemetry(stream.Telemetry{
		FPS:      frame.FPS,
		Score:    result.Score,
		Lost:     result.Lost,
		Tracking: trackingEnabled,
		PosX:     result.Point.X,
		PosY:     result.Point.Y,
	})

	return result
}

func (fp *FrameProcessor) renderAndEmit(frame camera.Frame, result TrackingResult, trackingEnabled bool, params FrameProcessorParams) {
	// Determine marker color
	markerColor := color.RGBA{G: 255} // Green - tracking OK
	if !trackingEnabled {
		markerColor = color.RGBA{R: 255, G: 255, B: 255} // White - disabled
	} else if result.Lost {
		markerColor = color.RGBA{R: 255} // Red - lost
	}

	// Create mirrored display frame
	display := frame.Mat.Clone()
	defer display.Close()
	gocv.Flip(display, &display, 1)

	// Mirror the point for display
	mirroredPoint := result.Point
	if display.Cols() > 0 {
		mirroredPoint = image.Point{X: display.Cols() - result.Point.X, Y: result.Point.Y}
	}

	// Draw overlay
	overlay.Draw(&display, overlay.Marker{
		Point: mirroredPoint,
		Shape: params.MarkerShape,
		Color: markerColor,
		Size:  params.TemplateSize,
		Lost:  result.Lost,
		Score: result.Score,
	})

	// Encode and emit preview
	if preview, ok := fp.preview.Encode(display); ok {
		fp.broker.EmitPreview(preview)
	}
}

// SetPickPoint sets a new tracking template at the given point.
// The point should be in mirrored (display) coordinates.
func (fp *FrameProcessor) SetPickPoint(displayPoint image.Point) error {
	fp.mu.RLock()
	frame := fp.lastFrame.Clone()
	fp.mu.RUnlock()
	defer frame.Close()

	if frame.Empty() {
		return ErrNoFrame
	}

	// Convert from mirrored display coordinates to frame coordinates
	point := displayPoint
	if frame.Cols() > 0 {
		point.X = frame.Cols() - displayPoint.X
	}

	err := fp.tracker.SetPickPoint(tracking.Frame{Mat: frame, Timestamp: time.Now()}, point)
	if err == nil {
		fp.mu.Lock()
		fp.markerPoint = point
		fp.markerValid = true
		fp.mu.Unlock()
	}

	return err
}

// Recenter sets the tracking template to the center of the frame.
func (fp *FrameProcessor) Recenter() error {
	fp.mu.RLock()
	frame := fp.lastFrame.Clone()
	fp.mu.RUnlock()
	defer frame.Close()

	if frame.Empty() {
		return ErrNoFrame
	}

	point := image.Point{X: frame.Cols() / 2, Y: frame.Rows() / 2}
	err := fp.tracker.SetPickPoint(tracking.Frame{Mat: frame, Timestamp: time.Now()}, point)
	if err == nil {
		fp.mu.Lock()
		fp.markerPoint = point
		fp.markerValid = true
		fp.mu.Unlock()
	}

	return err
}

// SetTrackingEnabled enables or disables tracking.
func (fp *FrameProcessor) SetTrackingEnabled(enabled bool) {
	fp.mu.Lock()
	fp.trackingEnabled = enabled
	fp.mu.Unlock()
}

// IsTrackingEnabled returns whether tracking is enabled.
func (fp *FrameProcessor) IsTrackingEnabled() bool {
	fp.mu.RLock()
	defer fp.mu.RUnlock()
	return fp.trackingEnabled
}

// SetTrackerParams updates the tracker parameters.
func (fp *FrameProcessor) SetTrackerParams(params tracking.Params) {
	fp.tracker.UpdateParams(params)
}

// SetProcessorParams updates the processor display parameters.
func (fp *FrameProcessor) SetProcessorParams(params FrameProcessorParams) {
	fp.mu.Lock()
	fp.params = params
	fp.mu.Unlock()
}

// Close releases resources held by the frame processor.
func (fp *FrameProcessor) Close() {
	fp.mu.Lock()
	defer fp.mu.Unlock()
	if !fp.lastFrame.Empty() {
		fp.lastFrame.Close()
	}
	fp.lastFrame = gocv.NewMat()
}

// Broker returns the stream broker for subscribing to events.
func (fp *FrameProcessor) Broker() *stream.Broker {
	return fp.broker
}
