package mouse

type Controller interface {
	Move(x, y int) error
	Click(button ClickButton) error
	CurrentPosition() (int, int, error)
}

type ClickButton string

const (
	ClickLeft   ClickButton = "left"
	ClickRight  ClickButton = "right"
	ClickMiddle ClickButton = "middle"
)
