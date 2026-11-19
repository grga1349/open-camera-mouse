//! A one-item mailbox between the capture thread and the app: the newest item replaces an
//! unread one (latest frame wins), and the reader waits with a timeout. A stalled camera
//! therefore shows up as a timeout on the reader side, even while the capture thread is
//! stuck inside a blocking backend call (ARCHITECTURE.md P5).

use std::sync::{Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Eq)]
pub enum Take<T> {
    Item(T),
    Timeout,
    /// Closed and empty.
    Closed,
}

#[derive(Debug)]
struct Inner<T> {
    item: Option<T>,
    closed: bool,
    /// Items replaced before anyone read them.
    dropped: u64,
}

#[derive(Debug)]
pub struct LatestSlot<T> {
    inner: Mutex<Inner<T>>,
    ready: Condvar,
}

impl<T> Default for LatestSlot<T> {
    fn default() -> Self {
        Self {
            inner: Mutex::new(Inner {
                item: None,
                closed: false,
                dropped: 0,
            }),
            ready: Condvar::new(),
        }
    }
}

impl<T> LatestSlot<T> {
    fn lock(&self) -> MutexGuard<'_, Inner<T>> {
        // A panic while holding this lock can't leave the two fields inconsistent.
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Stores `item`, replacing an unread one. Returns `false` (and drops `item`) once closed.
    pub fn put(&self, item: T) -> bool {
        let mut g = self.lock();
        if g.closed {
            return false;
        }
        if g.item.replace(item).is_some() {
            g.dropped += 1;
        }
        drop(g);
        self.ready.notify_one();
        true
    }

    /// Waits up to `timeout` for an item. An item stored before closing is still delivered.
    pub fn take_timeout(&self, timeout: Duration) -> Take<T> {
        let deadline = Instant::now() + timeout;
        let mut g = self.lock();
        loop {
            if let Some(item) = g.item.take() {
                return Take::Item(item);
            }
            if g.closed {
                return Take::Closed;
            }
            let now = Instant::now();
            if now >= deadline {
                return Take::Timeout;
            }
            g = self
                .ready
                .wait_timeout(g, deadline - now)
                .unwrap_or_else(|e| e.into_inner())
                .0;
        }
    }

    /// Rejects further `put`s and wakes a waiting reader.
    pub fn close(&self) {
        self.lock().closed = true;
        self.ready.notify_all();
    }

    pub fn is_closed(&self) -> bool {
        self.lock().closed
    }

    pub fn dropped(&self) -> u64 {
        self.lock().dropped
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    const SHORT: Duration = Duration::from_millis(20);

    #[test]
    fn newest_item_wins() {
        let s = LatestSlot::default();
        assert!(s.put(1));
        assert!(s.put(2));
        assert_eq!(s.take_timeout(SHORT), Take::Item(2));
        assert_eq!(s.dropped(), 1);
        assert_eq!(s.take_timeout(SHORT), Take::Timeout);
    }

    #[test]
    fn close_delivers_the_last_item_then_reports_closed() {
        let s = LatestSlot::default();
        s.put("err");
        s.close();
        assert!(!s.put("late"));
        assert_eq!(s.take_timeout(SHORT), Take::Item("err"));
        assert_eq!(s.take_timeout(SHORT), Take::Closed);
    }

    #[test]
    fn a_waiting_reader_wakes_on_put_and_on_close() {
        let s = Arc::new(LatestSlot::default());
        let w = {
            let s = s.clone();
            thread::spawn(move || s.take_timeout(Duration::from_secs(10)))
        };
        thread::sleep(SHORT);
        s.put(7);
        assert_eq!(w.join().unwrap(), Take::Item(7));

        let w = {
            let s = s.clone();
            thread::spawn(move || s.take_timeout(Duration::from_secs(10)))
        };
        thread::sleep(SHORT);
        s.close();
        assert_eq!(w.join().unwrap(), Take::<i32>::Closed);
    }

    #[test]
    fn timeout_is_respected() {
        let s = LatestSlot::<u8>::default();
        let t = Instant::now();
        assert_eq!(s.take_timeout(Duration::from_millis(50)), Take::Timeout);
        assert!(t.elapsed() >= Duration::from_millis(50));
    }
}
