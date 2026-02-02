//go:build windows

package hotkeys

import (
	"fmt"
	"runtime"
	"strings"
	"sync"
	"syscall"
	"unsafe"
)

var (
	user32                  = syscall.NewLazyDLL("user32.dll")
	procSetWindowsHookEx    = user32.NewProc("SetWindowsHookExW")
	procCallNextHookEx      = user32.NewProc("CallNextHookEx")
	procUnhookWindowsHookEx = user32.NewProc("UnhookWindowsHookEx")
	procGetMessage          = user32.NewProc("GetMessageW")
	procPostThreadMessage   = user32.NewProc("PostThreadMessageW")
)

const (
	whKeyboardLL = 13
	wmKeydown    = 0x0100
	wmQuit       = 0x0012
)

type kbdllHookStruct struct {
	VkCode      uint32
	ScanCode    uint32
	Flags       uint32
	Time        uint32
	DwExtraInfo uintptr
}

type msg struct {
	Hwnd    uintptr
	Message uint32
	WParam  uintptr
	LParam  uintptr
	Time    uint32
	Pt      struct{ X, Y int32 }
}

type manager struct {
	mu       sync.RWMutex
	bindings map[uint16]Action
	hook     uintptr
	threadID uint32
	started  bool
}

var (
	dispatchMu      sync.Mutex
	dispatchManager *manager
)

var (
	kernel32        = syscall.NewLazyDLL("kernel32.dll")
	procGetThreadID = kernel32.NewProc("GetCurrentThreadId")
)

func newManager() (Manager, error) {
	mgr := &manager{
		bindings: make(map[uint16]Action),
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

		m.threadID = getThreadID()

		dispatchMu.Lock()
		dispatchManager = m
		dispatchMu.Unlock()

		hook, _, err := procSetWindowsHookEx.Call(
			whKeyboardLL,
			syscall.NewCallback(lowLevelKeyboardProc),
			0,
			0,
		)
		if hook == 0 {
			ready <- fmt.Errorf("hotkeys: failed to set hook: %v", err)
			return
		}
		m.hook = hook
		ready <- nil

		var msg msg
		for {
			ret, _, _ := procGetMessage.Call(
				uintptr(unsafe.Pointer(&msg)),
				0, 0, 0,
			)
			if ret == 0 || int32(ret) == -1 {
				break
			}
		}
	}()

	if err := <-ready; err != nil {
		return err
	}
	m.started = true
	return nil
}

func lowLevelKeyboardProc(nCode int, wParam uintptr, lParam uintptr) uintptr {
	if nCode >= 0 && wParam == wmKeydown {
		kbStruct := (*kbdllHookStruct)(unsafe.Pointer(lParam))
		dispatchMu.Lock()
		mgr := dispatchManager
		dispatchMu.Unlock()
		if mgr != nil {
			mgr.dispatch(uint16(kbStruct.VkCode))
		}
	}
	ret, _, _ := procCallNextHookEx.Call(0, uintptr(nCode), wParam, lParam)
	return ret
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
		code, err := lookupWindowsKey(combo)
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
	if m.hook != 0 {
		procUnhookWindowsHookEx.Call(m.hook)
		m.hook = 0
	}
	if m.threadID != 0 {
		procPostThreadMessage.Call(uintptr(m.threadID), wmQuit, 0, 0)
	}
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

func getThreadID() uint32 {
	id, _, _ := procGetThreadID.Call()
	return uint32(id)
}

func lookupWindowsKey(input string) (uint16, error) {
	key := strings.ToUpper(strings.ReplaceAll(input, " ", ""))
	if key == "" {
		return 0, fmt.Errorf("hotkeys: empty key")
	}
	code, ok := windowsFunctionKeys[key]
	if !ok {
		return 0, fmt.Errorf("hotkeys: unsupported key %s", key)
	}
	return code, nil
}

var windowsFunctionKeys = map[string]uint16{
	"F1":  0x70,
	"F2":  0x71,
	"F3":  0x72,
	"F4":  0x73,
	"F5":  0x74,
	"F6":  0x75,
	"F7":  0x76,
	"F8":  0x77,
	"F9":  0x78,
	"F10": 0x79,
	"F11": 0x7A,
	"F12": 0x7B,
	"F13": 0x7C,
	"F14": 0x7D,
	"F15": 0x7E,
	"F16": 0x7F,
	"F17": 0x80,
	"F18": 0x81,
	"F19": 0x82,
	"F20": 0x83,
}
