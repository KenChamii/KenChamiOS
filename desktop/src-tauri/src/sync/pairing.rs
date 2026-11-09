use super::codes::{pairing_code, secrets_equal};
use crate::error::AppResult;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const LIFETIME: Duration = Duration::from_secs(5 * 60);
const MAX_FAILURES: u8 = 5;

#[derive(Debug, PartialEq, Eq)]
pub enum Attempt {
    Accepted,
    Rejected,
    /// no window is open: never opened, expired, used up or locked after too many tries
    Closed,
}

struct Open {
    code: String,
    expires: Instant,
    failures: u8,
}

/// The short time in which a new phone may pair; one code, one success, five wrong tries.
#[derive(Default)]
pub struct PairingWindow {
    open: Mutex<Option<Open>>,
}

impl PairingWindow {
    /// Opens a window with a fresh code, replacing an earlier one.
    pub fn open(&self, now: Instant) -> AppResult<(String, Instant)> {
        let code = pairing_code()?;
        let expires = now + LIFETIME;
        if let Ok(mut open) = self.open.lock() {
            *open = Some(Open {
                code: code.clone(),
                expires,
                failures: 0,
            });
        }
        Ok((code, expires))
    }

    pub fn close(&self) {
        if let Ok(mut open) = self.open.lock() {
            *open = None;
        }
    }

    pub fn is_open(&self, now: Instant) -> bool {
        self.open
            .lock()
            .is_ok_and(|open| open.as_ref().is_some_and(|w| w.expires > now))
    }

    pub fn attempt(&self, candidate: &str, now: Instant) -> Attempt {
        let Ok(mut guard) = self.open.lock() else {
            return Attempt::Closed;
        };
        let Some(window) = guard.as_mut() else {
            return Attempt::Closed;
        };
        if window.expires <= now {
            *guard = None;
            return Attempt::Closed;
        }
        if secrets_equal(&window.code, candidate) {
            *guard = None;
            return Attempt::Accepted;
        }
        window.failures += 1;
        if window.failures >= MAX_FAILURES {
            *guard = None;
        }
        Attempt::Rejected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_code_works_once() {
        let window = PairingWindow::default();
        let now = Instant::now();
        let (code, _) = window.open(now).expect("open");

        assert_eq!(window.attempt(&code, now), Attempt::Accepted);
        assert_eq!(window.attempt(&code, now), Attempt::Closed);
    }

    #[test]
    fn five_wrong_codes_close_the_window() {
        let window = PairingWindow::default();
        let now = Instant::now();
        let (code, _) = window.open(now).expect("open");

        for _ in 0..5 {
            assert_eq!(window.attempt("WRONGWRONG", now), Attempt::Rejected);
        }
        assert_eq!(window.attempt(&code, now), Attempt::Closed);
    }

    #[test]
    fn the_code_expires() {
        let window = PairingWindow::default();
        let now = Instant::now();
        let (code, expires) = window.open(now).expect("open");

        assert!(window.is_open(now));
        assert_eq!(window.attempt(&code, expires), Attempt::Closed);
        assert!(!window.is_open(expires));
    }

    #[test]
    fn nothing_pairs_while_closed() {
        let window = PairingWindow::default();
        assert_eq!(
            window.attempt("ANYTHING00", Instant::now()),
            Attempt::Closed
        );
    }

    #[test]
    fn a_new_window_replaces_the_old_code() {
        let window = PairingWindow::default();
        let now = Instant::now();
        let (old, _) = window.open(now).expect("open");
        let (new, _) = window.open(now).expect("open");

        assert_eq!(window.attempt(&old, now), Attempt::Rejected);
        assert_eq!(window.attempt(&new, now), Attempt::Accepted);
    }
}
