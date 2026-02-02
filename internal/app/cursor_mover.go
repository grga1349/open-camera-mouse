package app

import (
	"image"
	"math"
	"sync"

	"open-camera-mouse/internal/mouse"
)

type CursorMover struct {
	mu         sync.Mutex
	controller mouse.Controller
	mapper     *mouse.Mapper
	dwell      *mouse.DwellState

	lastPoint image.Point
	pointSet  bool
}

func NewCursorMover(controller mouse.Controller, mappingParams mouse.MappingParams, dwellParams mouse.DwellParams, onDwellClick func()) *CursorMover {
	cm := &CursorMover{
		controller: controller,
		mapper:     mouse.NewMapper(mappingParams),
	}
	cm.dwell = mouse.NewDwellState(controller, dwellParams, onDwellClick)
	return cm
}

func (cm *CursorMover) Update(point image.Point, lost bool) bool {
	cm.mu.Lock()
	defer cm.mu.Unlock()

	if lost {
		cm.mapper.Reset()
		cm.pointSet = false
		return false
	}

	if !cm.pointSet {
		cm.mapper.Reset()
		cm.pointSet = true
		cm.lastPoint = point
		return false
	}

	dx := float64(cm.lastPoint.X - point.X)
	dy := float64(point.Y - cm.lastPoint.Y)

	moveX, moveY := cm.mapper.Update(dx, dy)
	cm.lastPoint = point

	if moveX == 0 && moveY == 0 {
		return false
	}

	x, y, err := cm.controller.CurrentPosition()
	if err != nil {
		return false
	}

	targetX := int(math.Round(float64(x) + moveX))
	targetY := int(math.Round(float64(y) + moveY))
	_ = cm.controller.Move(targetX, targetY)

	return true
}

func (cm *CursorMover) UpdateDwell(lost bool) {
	if cm.dwell == nil {
		return
	}

	x, y, err := cm.controller.CurrentPosition()
	if err != nil {
		return
	}
	cm.dwell.Update(x, y, lost)
}

func (cm *CursorMover) Reset() {
	cm.mu.Lock()
	defer cm.mu.Unlock()
	cm.mapper.Reset()
	cm.pointSet = false
}

func (cm *CursorMover) SetMappingParams(params mouse.MappingParams) {
	cm.mu.Lock()
	defer cm.mu.Unlock()
	cm.mapper.SetParams(params)
}

func (cm *CursorMover) SetDwellParams(params mouse.DwellParams) {
	if cm.dwell != nil {
		cm.dwell.SetParams(params)
	}
}

func (cm *CursorMover) CenterCursor() {
	if cm.controller == nil {
		return
	}
	width, height, err := cm.controller.ScreenSize()
	if err != nil {
		return
	}
	_ = cm.controller.Move(width/2, height/2)
}
