package stream

import "sync"

type PreviewHandler func(PreviewFrame)
type TelemetryHandler func(Telemetry)

type Broker struct {
	mu sync.RWMutex

	previewHandlers   []PreviewHandler
	telemetryHandlers []TelemetryHandler
}

func NewBroker() *Broker {
	return &Broker{}
}

func (b *Broker) SubscribePreview(handler PreviewHandler) {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.previewHandlers = append(b.previewHandlers, handler)
}

func (b *Broker) SubscribeTelemetry(handler TelemetryHandler) {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.telemetryHandlers = append(b.telemetryHandlers, handler)
}

func (b *Broker) EmitPreview(frame PreviewFrame) {
	b.mu.RLock()
	handlers := append([]PreviewHandler(nil), b.previewHandlers...)
	b.mu.RUnlock()

	for _, handler := range handlers {
		handler(frame)
	}
}

func (b *Broker) EmitTelemetry(t Telemetry) {
	b.mu.RLock()
	handlers := append([]TelemetryHandler(nil), b.telemetryHandlers...)
	b.mu.RUnlock()

	for _, handler := range handlers {
		handler(t)
	}
}
