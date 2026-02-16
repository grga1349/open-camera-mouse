//go:build darwin

package camera

import "os"

func init() {
	// Prevent OpenCV from selecting FFMPEG camera backend on macOS.
	// The packaged app has shown runtime aborts in the FFMPEG path.
	_ = os.Setenv("OPENCV_VIDEOIO_PRIORITY_FFMPEG", "0")
	_ = os.Setenv("OPENCV_VIDEOIO_PRIORITY_AVFOUNDATION", "1000")
}

