//go:build darwin

package hotkeys

/*
#cgo darwin CFLAGS: -x objective-c -fmodules -fobjc-arc
#cgo darwin LDFLAGS: -framework Cocoa -framework Carbon
#include <ApplicationServices/ApplicationServices.h>
#include <stdbool.h>

static CFMachPortRef hotkeyTap = NULL;
static CFRunLoopSourceRef hotkeySource = NULL;
static CFRunLoopRef hotkeyLoop = NULL;

extern void hotkeyDispatch(unsigned short code);

static CGEventRef hotkeyCallback(CGEventTapProxy proxy, CGEventType type, CGEventRef event, void *refcon) {
	if (type == kCGEventTapDisabledByTimeout || type == kCGEventTapDisabledByUserInput) {
		if (hotkeyTap != NULL) {
			CGEventTapEnable(hotkeyTap, true);
		}
		return event;
	}
	if (type != kCGEventKeyDown) {
		return event;
	}
	CGKeyCode code = (CGKeyCode)CGEventGetIntegerValueField(event, kCGKeyboardEventKeycode);
	hotkeyDispatch((unsigned short)code);
	return event;
}

static bool startHotkeyTap() {
	if (hotkeyTap != NULL) {
		return true;
	}
	hotkeyTap = CGEventTapCreate(kCGSessionEventTap, kCGHeadInsertEventTap, kCGEventTapOptionListenOnly,
		CGEventMaskBit(kCGEventKeyDown), hotkeyCallback, NULL);
	if (hotkeyTap == NULL) {
		return false;
	}
	hotkeySource = CFMachPortCreateRunLoopSource(kCFAllocatorDefault, hotkeyTap, 0);
	hotkeyLoop = CFRunLoopGetCurrent();
	CFRunLoopAddSource(hotkeyLoop, hotkeySource, kCFRunLoopCommonModes);
	CGEventTapEnable(hotkeyTap, true);
	return true;
}

static void stopHotkeyTap() {
	if (hotkeyTap != NULL) {
		CGEventTapEnable(hotkeyTap, false);
		CFRunLoopRemoveSource(hotkeyLoop, hotkeySource, kCFRunLoopCommonModes);
		CFRelease(hotkeySource);
		CFRelease(hotkeyTap);
		hotkeySource = NULL;
		hotkeyTap = NULL;
	}
	if (hotkeyLoop != NULL) {
		CFRunLoopStop(hotkeyLoop);
		hotkeyLoop = NULL;
	}
}
*/
import "C"

import (
	"fmt"
	"runtime"
	"strings"
	"sync"
)

type manager struct {
	mu       sync.RWMutex
	bindings map[uint16]Action
	started  bool
}

var (
	dispatchMu      sync.Mutex
	dispatchManager *manager
)

func newManager() (Manager, error) {
	mgr := &manager{bindings: make(map[uint16]Action)}
	if err := mgr.start(); err != nil {
		return nil, err
	}
	return mgr, nil
}

func (m *manager) start() error {
	ready := make(chan error, 1)
	go func() {
		runtime.LockOSThread()
		ok := bool(C.startHotkeyTap())
		if !ok {
			ready <- fmt.Errorf("hotkeys: failed to start event tap")
			runtime.UnlockOSThread()
			return
		}
		dispatchMu.Lock()
		dispatchManager = m
		dispatchMu.Unlock()
		ready <- nil
		C.CFRunLoopRun()
		runtime.UnlockOSThread()
	}()
	if err := <-ready; err != nil {
		return err
	}
	m.started = true
	return nil
}

func (m *manager) Update(bindings map[string]Action) error {
	m.mu.Lock()
	defer m.mu.Unlock()
	updated := make(map[uint16]Action)
	for combo, action := range bindings {
		combo = strings.TrimSpace(combo)
		if combo == "" || action == nil {
			continue
		}
		code, err := lookupMacKey(combo)
		if err != nil {
			return err
		}
		updated[code] = action
	}
	m.bindings = updated
	return nil
}

func (m *manager) Close() {
	if !m.started {
		return
	}
	C.stopHotkeyTap()
	dispatchMu.Lock()
	if dispatchManager == m {
		dispatchManager = nil
	}
	dispatchMu.Unlock()
	m.started = false
}

func (m *manager) dispatch(code uint16) {
	m.mu.RLock()
	action := m.bindings[code]
	m.mu.RUnlock()
	if action != nil {
		go action()
	}
}

//export hotkeyDispatch
func hotkeyDispatch(code C.ushort) {
	dispatchMu.Lock()
	manager := dispatchManager
	dispatchMu.Unlock()
	if manager != nil {
		manager.dispatch(uint16(code))
	}
}

func lookupMacKey(input string) (uint16, error) {
	key := strings.ToUpper(strings.ReplaceAll(input, " ", ""))
	if key == "" {
		return 0, fmt.Errorf("hotkeys: empty key")
	}
	code, ok := macFunctionKeys[key]
	if !ok {
		return 0, fmt.Errorf("hotkeys: unsupported key %s", key)
	}
	return code, nil
}

var macFunctionKeys = map[string]uint16{
	"F1":  122,
	"F2":  120,
	"F3":  99,
	"F4":  118,
	"F5":  96,
	"F6":  97,
	"F7":  98,
	"F8":  100,
	"F9":  101,
	"F10": 109,
	"F11": 103,
	"F12": 111,
	"F13": 105,
	"F14": 107,
	"F15": 113,
	"F16": 106,
	"F17": 64,
	"F18": 79,
	"F19": 80,
	"F20": 90,
}
