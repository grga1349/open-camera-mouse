package overlay

import (
	"fmt"
	"image"
	"image/color"

	"gocv.io/x/gocv"
)

type Marker struct {
	Point image.Point
	Shape string
	Color color.RGBA
	Size  int
	Lost  bool
	Score float32
}

func Draw(frame *gocv.Mat, marker Marker) {
	if frame == nil || frame.Empty() {
		return
	}

	switch marker.Shape {
	case "square":
		drawSquare(frame, marker)
	default:
		drawCircle(frame, marker)
	}

	drawScore(frame, marker)
}

func drawCircle(frame *gocv.Mat, marker Marker) {
	radius := marker.Size / 2
	gocv.Circle(frame, marker.Point, radius, marker.Color, 2)
}

func drawSquare(frame *gocv.Mat, marker Marker) {
	half := marker.Size / 2
	rect := image.Rect(marker.Point.X-half, marker.Point.Y-half, marker.Point.X+half, marker.Point.Y+half)
	gocv.Rectangle(frame, rect, marker.Color, 2)
}

func drawScore(frame *gocv.Mat, marker Marker) {
	status := "OK"
	if marker.Lost {
		status = "LOST"
	}
	text := fmt.Sprintf("%s %.2f", status, marker.Score)
	gocv.PutText(frame, text, image.Pt(10, 30), gocv.FontHersheyPlain, 1.2, marker.Color, 2)
}
