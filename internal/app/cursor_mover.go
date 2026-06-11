package app

import (
	"image"
	"math"
	"sync"

	"open-camera-mouse/internal/mouse"
)

type CursorMover struct {
	controller mouse.Controller
	mapper     *mouse.Mapper
	dwell      *mouse.DwellState

	// pipeline-goroutine-owned — no lock needed
	lastPoint image.Point
	pointSet  bool

	// control-plane → pipeline dirty flags
	mappingMu      sync.Mutex
	pendingMapping mouse.MappingParams
	mappingDirty   bool
	resetPending   bool
}

func NewCursorMover(
	controller mouse.Controller,
	mappingParams mouse.MappingParams,
	dwellParams mouse.DwellParams,
	onDwellClick func(),
) *CursorMover {
	cm := &CursorMover{
		controller: controller,
		mapper:     mouse.NewMapper(mappingParams),
	}
	cm.dwell = mouse.NewDwellState(controller, dwellParams, onDwellClick)
	return cm
}

func (cm *CursorMover) Update(point image.Point, lost bool) bool {
	cm.mappingMu.Lock()
	pending, dirty, reset := cm.pendingMapping, cm.mappingDirty, cm.resetPending
	cm.mappingDirty, cm.resetPending = false, false
	cm.mappingMu.Unlock()

	if reset {
		cm.mapper.Reset()
		cm.pointSet = false
	}
	if dirty {
		cm.mapper.SetParams(pending)
	}

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
	cm.mappingMu.Lock()
	cm.resetPending = true
	cm.mappingMu.Unlock()
}

func (cm *CursorMover) SetMappingParams(params mouse.MappingParams) {
	cm.mappingMu.Lock()
	cm.pendingMapping = params
	cm.mappingDirty = true
	cm.mappingMu.Unlock()
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
