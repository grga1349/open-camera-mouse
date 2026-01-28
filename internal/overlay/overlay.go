package overlay

// Renderer will draw tracking overlays onto preview frames.
type Renderer struct{}

func NewRenderer() *Renderer {
	return &Renderer{}
}
