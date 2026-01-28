package mouse

// Controller abstracts OS mouse input.
type Controller struct{}

func NewController() *Controller {
	return &Controller{}
}
