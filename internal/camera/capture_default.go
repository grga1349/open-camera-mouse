//go:build !darwin

package camera

import "gocv.io/x/gocv"

func openVideoCapture(deviceID int) (*gocv.VideoCapture, error) {
	return gocv.VideoCaptureDevice(deviceID)
}

