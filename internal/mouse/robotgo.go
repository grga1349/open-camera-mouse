package mouse

import (
	"fmt"

	"github.com/go-vgo/robotgo"
)

type RobotController struct{}

func NewRobotController() *RobotController {
	return &RobotController{}
}

func (r *RobotController) Move(x, y int) error {
	robotgo.Move(x, y)
	return nil
}

func (r *RobotController) Click(button ClickButton) error {
	switch button {
	case ClickLeft:
		robotgo.Click("left", false)
	case ClickRight:
		robotgo.Click("right", false)
	case ClickMiddle:
		robotgo.Click("center", false)
	default:
		return fmt.Errorf("mouse: unsupported button %s", button)
	}
	return nil
}

func (r *RobotController) CurrentPosition() (int, int, error) {
	x, y := robotgo.GetMousePos()
	return x, y, nil
}

func (r *RobotController) ScreenSize() (int, int, error) {
	w, h := robotgo.GetScreenSize()
	return w, h, nil
}
