package stream

type Telemetry struct {
	FPS      float64 `json:"fps"`
	Score    float32 `json:"score"`
	Lost     bool    `json:"lost"`
	Tracking bool    `json:"tracking"`
	PosX     int     `json:"posX"`
	PosY     int     `json:"posY"`
}
