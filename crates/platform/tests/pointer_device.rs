//! On-device pointer checks (task 0008 D4, ARCHITECTURE.md P6, R-P5). Ignored by default: it
//! moves the real cursor, and CI has no desktop session. Run on a machine with a display and,
//! on macOS, Accessibility granted to the terminal:
//!
//! ```text
//! cargo test -p platform --test pointer_device -- --ignored --nocapture --test-threads=1
//! ```
//!
//! Doesn't click (a click would land on whatever is under the cursor): dwell clicking is in the
//! D7 manual QA. Leave the mouse alone while it runs (about a second).

use std::thread::sleep;
use std::time::Duration;

use app_core::ports::{Point2, PointerAccess, PointerSink};
use platform::SystemPointer;

/// Generous: posted moves land in under 1 ms typically, ~38 ms worst measured.
const SETTLE: Duration = Duration::from_millis(80);

fn near(a: Point2, b: Point2) -> bool {
    (a.x - b.x).hypot(a.y - b.y) <= 1.5
}

/// The raw OS cursor, read through a fresh adapter that has caused no moves.
fn raw() -> Point2 {
    SystemPointer::new()
        .expect("pointer")
        .position()
        .expect("position")
}

#[test]
#[ignore = "moves the real cursor; needs a desktop session (and Accessibility on macOS)"]
fn moves_land_and_read_back_as_requested() {
    let mut ptr = SystemPointer::new().expect("pointer");
    let access = ptr.access();
    println!("access: {access:?}");
    assert_eq!(
        access,
        PointerAccess::Granted,
        "grant Accessibility to the terminal running the test"
    );
    let desktop = ptr.desktop().expect("desktop").bounds;
    println!("desktop: {desktop:?}");
    let start = raw();

    // On a display: lands exactly (within OS rounding), reads back as requested.
    let c = desktop.center();
    let mut targets = vec![Point2::new(desktop.x + 10.0, desktop.y + 10.0), c];
    // The far corner, if it is on a display.
    targets.push(Point2::new(
        desktop.x + desktop.width - 11.0,
        desktop.y + desktop.height - 11.0,
    ));
    for t in targets {
        ptr.move_to(t).expect("move");
        sleep(SETTLE);
        let r = raw();
        let reported = ptr.position().expect("position");
        println!("target {t:?} raw {r:?} reported {reported:?}");
        assert_eq!(
            reported, t,
            "the adapter's own move must read back as requested"
        );
    }

    // Outside every display: the cursor lands on a display, still reads back as requested.
    let outside = Point2::new(desktop.x - 500.0, desktop.y - 500.0);
    ptr.move_to(outside).expect("move");
    sleep(SETTLE);
    let r = raw();
    println!("outside {outside:?} raw {r:?}");
    assert!(
        r.x >= desktop.x && r.y >= desktop.y,
        "snapped onto the desktop"
    );
    assert_eq!(ptr.position(), Some(outside));

    // Non-finite targets are refused before reaching the OS.
    assert!(ptr.move_to(Point2::new(f32::NAN, 0.0)).is_err());

    ptr.move_to(start).expect("restore");
    sleep(SETTLE);
    assert!(near(raw(), start), "cursor restored");
}
