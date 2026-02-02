//go:build linux

package hotkeys

/*
#cgo linux pkg-config: x11
#include <X11/Xlib.h>
#include <X11/keysym.h>
#include <stdlib.h>

static Display* openDisplay() {
	return XOpenDisplay(NULL);
}

static void closeDisplay(Display* d) {
	if (d != NULL) {
		XCloseDisplay(d);
	}
}

static Window getRoot(Display* d) {
	return DefaultRootWindow(d);
}

static int grabKey(Display* d, Window root, unsigned int keycode) {
	return XGrabKey(d, keycode, AnyModifier, root, True, GrabModeAsync, GrabModeAsync);
}

static void ungrabKey(Display* d, Window root, unsigned int keycode) {
	XUngrabKey(d, keycode, AnyModifier, root);
}

static void ungrabAll(Display* d, Window root) {
	XUngrabKey(d, AnyKey, AnyModifier, root);
}

static int pendingEvents(Display* d) {
	return XPending(d);
}

static int nextEvent(Display* d, XEvent* ev) {
	return XNextEvent(d, ev);
}

static int eventType(XEvent* ev) {
	return ev->type;
}

static unsigned int eventKeycode(XEvent* ev) {
	return ev->xkey.keycode;
}

static unsigned int keysymToKeycode(Display* d, unsigned long keysym) {
	return XKeysymToKeycode(d, keysym);
}
*/
import "C"

import (
	"fmt"
	"runtime"
	"strings"
	"sync"
	"time"
)

type manager struct {
	mu        sync.RWMutex
	bindings  map[uint]Action
	keycodes  map[uint]bool
	display   *C.Display
	root      C.Window
	started   bool
	stopCh    chan struct{}
	closeOnce sync.Once
}

var (
	dispatchMu      sync.Mutex
	dispatchManager *manager
)

func newManager() (Manager, error) {
	mgr := &manager{
		bindings: make(map[uint]Action),
		keycodes: make(map[uint]bool),
		stopCh:   make(chan struct{}),
	}
	if err := mgr.start(); err != nil {
		return nil, err
	}
	return mgr, nil
}

func (m *manager) start() error {
	ready := make(chan error, 1)

	go func() {
		runtime.LockOSThread()
		defer runtime.UnlockOSThread()

		display := C.openDisplay()
		if display == nil {
			ready <- fmt.Errorf("hotkeys: failed to open X display")
			return
		}
		m.display = display
		m.root = C.getRoot(display)

		dispatchMu.Lock()
		dispatchManager = m
		dispatchMu.Unlock()

		ready <- nil
		m.eventLoop()
	}()

	if err := <-ready; err != nil {
		return err
	}
	m.started = true
	return nil
}

func (m *manager) eventLoop() {
	var event C.XEvent

	for {
		select {
		case <-m.stopCh:
			return
		default:
		}

		if C.pendingEvents(m.display) > 0 {
			C.nextEvent(m.display, &event)
			evType := C.eventType(&event)
			if evType == C.KeyPress {
				keycode := uint(C.eventKeycode(&event))
				m.dispatch(keycode)
			}
		} else {
			time.Sleep(10 * time.Millisecond)
		}
	}
}

func (m *manager) Update(bindings map[string]Action) error {
	m.mu.Lock()
	defer m.mu.Unlock()

	for keycode := range m.keycodes {
		C.ungrabKey(m.display, m.root, C.uint(keycode))
	}
	m.keycodes = make(map[uint]bool)
	m.bindings = make(map[uint]Action)

	for combo, action := range bindings {
		combo = strings.TrimSpace(combo)
		if combo == "" || action == nil {
			continue
		}
		keysym, err := lookupLinuxKeysym(combo)
		if err != nil {
			return err
		}
		keycode := uint(C.keysymToKeycode(m.display, C.ulong(keysym)))
		if keycode == 0 {
			return fmt.Errorf("hotkeys: failed to get keycode for %s", combo)
		}
		C.grabKey(m.display, m.root, C.uint(keycode))
		m.bindings[keycode] = action
		m.keycodes[keycode] = true
	}

	return nil
}

func (m *manager) Close() {
	if !m.started {
		return
	}

	m.closeOnce.Do(func() {
		close(m.stopCh)
	})

	m.mu.Lock()
	if m.display != nil {
		C.ungrabAll(m.display, m.root)
		C.closeDisplay(m.display)
		m.display = nil
	}
	m.mu.Unlock()

	dispatchMu.Lock()
	if dispatchManager == m {
		dispatchManager = nil
	}
	dispatchMu.Unlock()
	m.started = false
}

func (m *manager) dispatch(keycode uint) {
	m.mu.RLock()
	action := m.bindings[keycode]
	m.mu.RUnlock()
	if action != nil {
		go action()
	}
}

func lookupLinuxKeysym(input string) (uint, error) {
	key := strings.ToUpper(strings.ReplaceAll(input, " ", ""))
	if key == "" {
		return 0, fmt.Errorf("hotkeys: empty key")
	}
	keysym, ok := linuxFunctionKeys[key]
	if !ok {
		return 0, fmt.Errorf("hotkeys: unsupported key %s", key)
	}
	return keysym, nil
}

var linuxFunctionKeys = map[string]uint{
	"F1":  0xFFBE,
	"F2":  0xFFBF,
	"F3":  0xFFC0,
	"F4":  0xFFC1,
	"F5":  0xFFC2,
	"F6":  0xFFC3,
	"F7":  0xFFC4,
	"F8":  0xFFC5,
	"F9":  0xFFC6,
	"F10": 0xFFC7,
	"F11": 0xFFC8,
	"F12": 0xFFC9,
	"F13": 0xFFCA,
	"F14": 0xFFCB,
	"F15": 0xFFCC,
	"F16": 0xFFCD,
	"F17": 0xFFCE,
	"F18": 0xFFCF,
	"F19": 0xFFD0,
	"F20": 0xFFD1,
}
