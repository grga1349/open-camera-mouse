package stream

import (
	"context"
	"sync"
)

type BrokerPolicy struct {
	PreviewBuffer        int
	TelemetryBuffer      int
	DropPreviewIfSlow    bool
	DropTelemetryIfSlow  bool
}

func DefaultBrokerPolicy() BrokerPolicy {
	return BrokerPolicy{
		PreviewBuffer:       4,
		TelemetryBuffer:     4,
		DropPreviewIfSlow:   true,
		DropTelemetryIfSlow: false,
	}
}

type Broker struct {
	policy   BrokerPolicy
	mu       sync.Mutex
	previews []chan PreviewFrame
	telems   []chan Telemetry
}

func NewBroker(policy BrokerPolicy) *Broker {
	return &Broker{policy: policy}
}

func (b *Broker) SubscribePreview(ctx context.Context, buffer int) <-chan PreviewFrame {
	ch := make(chan PreviewFrame, buffer)
	b.mu.Lock()
	b.previews = append(b.previews, ch)
	b.mu.Unlock()
	go func() {
		<-ctx.Done()
		b.mu.Lock()
		b.previews = removePreviewCh(b.previews, ch)
		b.mu.Unlock()
		close(ch)
	}()
	return ch
}

func (b *Broker) SubscribeTelemetry(ctx context.Context, buffer int) <-chan Telemetry {
	ch := make(chan Telemetry, buffer)
	b.mu.Lock()
	b.telems = append(b.telems, ch)
	b.mu.Unlock()
	go func() {
		<-ctx.Done()
		b.mu.Lock()
		b.telems = removeTelemetryCh(b.telems, ch)
		b.mu.Unlock()
		close(ch)
	}()
	return ch
}

func (b *Broker) PublishPreview(frame PreviewFrame) {
	b.mu.Lock()
	subs := append([]chan PreviewFrame(nil), b.previews...)
	b.mu.Unlock()
	for _, ch := range subs {
		if b.policy.DropPreviewIfSlow {
			select {
			case ch <- frame:
			default:
			}
		} else {
			ch <- frame
		}
	}
}

func (b *Broker) PublishTelemetry(t Telemetry) {
	b.mu.Lock()
	subs := append([]chan Telemetry(nil), b.telems...)
	b.mu.Unlock()
	for _, ch := range subs {
		if b.policy.DropTelemetryIfSlow {
			select {
			case ch <- t:
			default:
			}
		} else {
			ch <- t
		}
	}
}

func removePreviewCh(s []chan PreviewFrame, ch chan PreviewFrame) []chan PreviewFrame {
	out := s[:0]
	for _, c := range s {
		if c != ch {
			out = append(out, c)
		}
	}
	return out
}

func removeTelemetryCh(s []chan Telemetry, ch chan Telemetry) []chan Telemetry {
	out := s[:0]
	for _, c := range s {
		if c != ch {
			out = append(out, c)
		}
	}
	return out
}
