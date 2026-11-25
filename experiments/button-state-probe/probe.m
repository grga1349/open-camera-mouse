#import <ApplicationServices/ApplicationServices.h>
#import <Cocoa/Cocoa.h>
#include <stdio.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

typedef struct {
    bool hid;
    bool session;
} ButtonState;

static ButtonState sample(const char *label, double start) {
    ButtonState state = {
        CGEventSourceButtonState(kCGEventSourceStateHIDSystemState, kCGMouseButtonLeft),
        CGEventSourceButtonState(kCGEventSourceStateCombinedSessionState, kCGMouseButtonLeft),
    };
    double elapsed = (NSProcessInfo.processInfo.systemUptime - start) * 1000.0;
    printf("%8.1f ms  %-26s HID=%d  session=%d\n", elapsed, label, state.hid, state.session);
    return state;
}

static void after_ms(int delay, dispatch_block_t block) {
    dispatch_after(dispatch_time(DISPATCH_TIME_NOW, (int64_t)delay * NSEC_PER_MSEC),
                   dispatch_get_main_queue(), block);
}

@interface ProbeView : NSView <NSWindowDelegate>
@end

@implementation ProbeView {
    NSString *_instruction;
    double _start;
    double _physicalPress;
    bool _physicalDown;
    bool _physicalControlPassed;
    bool _armed;
    bool _finished;
    bool _interference;
    bool _taggedDown;
    bool _taggedUp;
    bool _upPosted;
    bool _hidDownBeforeConfirmation;
    int _physicalPositiveSamples;
    int _syntheticHidDown;
    int _syntheticHidUp;
    int64_t _tag;
}

- (instancetype)initWithFrame:(NSRect)frame {
    self = [super initWithFrame:frame];
    if (self) {
        _start = NSProcessInfo.processInfo.systemUptime;
        _tag = INT64_C(0x4f434d0000000000) | getpid();
        _instruction = @"Hold the physical left button here for one second, then release.";
    }
    return self;
}

- (void)showInstruction:(NSString *)instruction {
    _instruction = instruction;
    self.needsDisplay = YES;
}

- (void)drawRect:(NSRect)dirtyRect {
    [NSColor.windowBackgroundColor setFill];
    NSRectFill(dirtyRect);
    NSDictionary *attributes = @{
        NSFontAttributeName: [NSFont systemFontOfSize:15],
        NSForegroundColorAttributeName: NSColor.labelColor,
    };
    [_instruction drawWithRect:NSInsetRect(self.bounds, 24, 24)
                       options:NSStringDrawingUsesLineFragmentOrigin
                    attributes:attributes];
}

- (BOOL)acceptsFirstResponder {
    return YES;
}

- (BOOL)windowShouldClose:(NSWindow *)window {
    (void)window;
    return !_armed;
}

- (void)windowWillClose:(NSNotification *)notification {
    (void)notification;
    [NSApp terminate:nil];
}

- (void)windowDidResignKey:(NSNotification *)notification {
    (void)notification;
    if (_armed) {
        _interference = true;
    }
}

- (void)mouseMoved:(NSEvent *)event {
    (void)event;
    if (_armed) {
        _interference = true;
    }
}

- (void)mouseDragged:(NSEvent *)event {
    [self mouseMoved:event];
}

- (bool)isTagged:(NSEvent *)event {
    CGEventRef quartzEvent = event.CGEvent;
    return quartzEvent && CGEventGetIntegerValueField(quartzEvent, kCGEventSourceUserData) == _tag;
}

- (void)mouseDown:(NSEvent *)event {
    if ([self isTagged:event]) {
        if (_armed && !_upPosted) {
            _taggedDown = true;
            [self sampleSyntheticHold:"tagged down delivered"];
        }
        return;
    }
    if (_armed) {
        _interference = true;
        return;
    }
    if (_finished || _physicalControlPassed) {
        return;
    }

    _physicalDown = true;
    _physicalPress = event.timestamp;
    _physicalPositiveSamples = sample("physical down delivered", _start).hid ? 1 : 0;
    double press = _physicalPress;
    for (int delay = 100; delay <= 300; delay += 200) {
        after_ms(delay, ^{
            if (self->_physicalDown && self->_physicalPress == press &&
                sample("physical held", self->_start).hid) {
                self->_physicalPositiveSamples++;
            }
        });
    }
}

- (void)mouseUp:(NSEvent *)event {
    if ([self isTagged:event]) {
        if (_armed && _upPosted) {
            _taggedUp = true;
            sample("tagged up delivered", _start);
        }
        return;
    }
    if (_armed) {
        _interference = true;
        return;
    }
    if (!_physicalDown) {
        return;
    }
    _physicalDown = false;
    sample("physical up delivered", _start);
    _physicalControlPassed = _physicalPositiveSamples >= 2;
    if (_physicalControlPassed) {
        [self showInstruction:@"Leave the pointer here. Press Space, then keep your hands off the mouse."];
    } else {
        [self showInstruction:@"Physical control incomplete. Hold the left button here for one second."];
    }
}

- (void)keyDown:(NSEvent *)event {
    if (_armed || _finished || !_physicalControlPassed ||
        ![event.charactersIgnoringModifiers isEqualToString:@" "]) {
        return;
    }
    _armed = true;
    [self showInstruction:@"Posting in two seconds. Keep the pointer here and leave all buttons released."];
    after_ms(2000, ^{
        [self runSyntheticHold];
    });
}

- (void)finish:(const char *)verdict {
    _armed = false;
    _finished = true;
    printf("%s\n", verdict);
    [self showInstruction:@"Finished. Close this window and read the result in Terminal."];
}

- (void)sampleSyntheticHold:(const char *)label {
    ButtonState state = sample(label, _start);
    if (_upPosted) {
        return;
    }
    if (!_taggedDown) {
        _hidDownBeforeConfirmation |= state.hid;
    } else if (state.hid) {
        _syntheticHidDown++;
    } else {
        _syntheticHidUp++;
    }
}

- (void)runSyntheticHold {
    ButtonState baseline = sample("before synthetic down", _start);
    CGEventRef cursor = CGEventCreate(NULL);
    if (!cursor) {
        [self finish:"INCONCLUSIVE: could not locate the pointer."];
        return;
    }
    // Read the same captured pointer in AppKit's and Quartz's global coordinate spaces.
    CGPoint pointer = CGEventGetUnflippedLocation(cursor);
    CGPoint location = CGEventGetLocation(cursor);
    CFRelease(cursor);
    NSInteger hitWindow = [NSWindow windowNumberAtPoint:pointer belowWindowWithWindowNumber:0];
    NSPoint localPointer = [self convertPoint:[self.window convertPointFromScreen:pointer]
                                     fromView:nil];
    if (_interference || baseline.hid || baseline.session || NSEvent.pressedMouseButtons != 0 ||
        !NSApp.active ||
        !self.window.keyWindow || hitWindow != self.window.windowNumber ||
        !NSPointInRect(localPointer, NSInsetRect(self.bounds, 15, 15))) {
        [self finish:"INCONCLUSIVE: release all buttons and leave the pointer within the active probe window."];
        return;
    }

    CGEventSourceRef source = CGEventSourceCreate(kCGEventSourceStateHIDSystemState);
    if (!source) {
        [self finish:"INCONCLUSIVE: CoreGraphics could not create the event source."];
        return;
    }
    CGEventSourceSetLocalEventsSuppressionInterval(source, 0.0);
    CGEventSourceSetUserData(source, _tag);
    CGEventRef down = CGEventCreateMouseEvent(source, kCGEventLeftMouseDown, location, kCGMouseButtonLeft);
    CGEventRef up = CGEventCreateMouseEvent(source, kCGEventLeftMouseUp, location, kCGMouseButtonLeft);
    if (!down || !up) {
        if (down) {
            CFRelease(down);
        }
        if (up) {
            CFRelease(up);
        }
        CFRelease(source);
        [self finish:"INCONCLUSIVE: CoreGraphics could not create the paired mouse events."];
        return;
    }

    // Reserve the matching up before posting the down. Both use the same source and location.
    after_ms(750, ^{
        self->_upPosted = true;
        CGEventSetTimestamp(up, clock_gettime_nsec_np(CLOCK_UPTIME_RAW));
        CGEventPost(kCGHIDEventTap, up);
        CFRelease(up);
        CFRelease(source);
        sample("immediately after up", self->_start);
    });
    CGEventPost(kCGHIDEventTap, down);
    CFRelease(down);
    [self sampleSyntheticHold:"immediately after down"];
    const int holdDelays[] = {25, 100, 250, 500};
    for (size_t i = 0; i < sizeof(holdDelays) / sizeof(holdDelays[0]); i++) {
        after_ms(holdDelays[i], ^{
            [self sampleSyntheticHold:self->_taggedDown ? "confirmed hold sample" : "delivery pending sample"];
        });
    }
    after_ms(850, ^{
        sample("delayed after up", self->_start);
    });
    after_ms(1250, ^{
        ButtonState final = sample("settled after up", self->_start);
        if (self->_interference || !self->_taggedDown || !self->_taggedUp || final.hid || final.session) {
            [self finish:"INCONCLUSIVE: delivery, clean input, or recovery was unconfirmed."];
        } else if (self->_syntheticHidDown > 0) {
            [self finish:"INCLUDES: HID reported down during a confirmed synthetic hold."];
        } else if (self->_hidDownBeforeConfirmation) {
            [self finish:"INCONCLUSIVE: HID reported down before delivery was confirmed."];
        } else if (self->_syntheticHidUp >= 3) {
            [self finish:"EXCLUDES IN OBSERVED SAMPLES: HID stayed up during the confirmed synthetic hold."];
        } else {
            [self finish:"INCONCLUSIVE: insufficient samples during the confirmed synthetic hold."];
        }
    });
}

@end

int main(int argc, const char *argv[]) {
    if (argc != 2 || strcmp(argv[1], "--run") != 0) {
        puts("A3 macOS button-state spike. Read README.md, then pass --run to open the probe.");
        return argc == 2 && strcmp(argv[1], "--help") == 0 ? 0 : 2;
    }
    setvbuf(stdout, NULL, _IOLBF, 0);
    @autoreleasepool {
        printf("macOS %s\n", NSProcessInfo.processInfo.operatingSystemVersionString.UTF8String);
        bool preflight = CGPreflightPostEventAccess();
        bool trusted = AXIsProcessTrusted();
        printf("Posting permission: preflight=%d AX trusted=%d\n", preflight, trusted);
        if (!preflight && !trusted) {
            fputs("Grant Accessibility to the launching terminal and retry. See README.md.\n", stderr);
            return 1;
        }

        [NSApplication sharedApplication];
        [NSApp setActivationPolicy:NSApplicationActivationPolicyRegular];
        NSRect frame = NSMakeRect(0, 0, 510, 150);
        __attribute__((objc_precise_lifetime)) NSWindow *window =
            [[NSWindow alloc] initWithContentRect:frame
                                       styleMask:NSWindowStyleMaskTitled | NSWindowStyleMaskClosable
                                         backing:NSBackingStoreBuffered
                                           defer:NO];
        ProbeView *view = [[ProbeView alloc] initWithFrame:frame];
        window.title = @"Open Camera Mouse · A3 button-state probe";
        window.contentView = view;
        window.delegate = view;
        window.acceptsMouseMovedEvents = YES;
        window.releasedWhenClosed = NO;
        [window center];
        [window makeFirstResponder:view];
        [window makeKeyAndOrderFront:nil];
        [NSRunningApplication.currentApplication activateWithOptions:0];
        puts("Hold the physical left button in the probe window for one second, then release.");
        [NSApp run];
    }
    return 0;
}
