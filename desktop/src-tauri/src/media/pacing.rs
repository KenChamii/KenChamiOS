use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Time source and sleeper, so the spacing can be tested without waiting.
pub trait Clock {
    /// Time since an arbitrary fixed start; it never goes backwards.
    fn elapsed(&self) -> Duration;
    async fn sleep(&self, duration: Duration);
}

pub struct SystemClock {
    start: Instant,
}

impl Default for SystemClock {
    fn default() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl Clock for SystemClock {
    fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    async fn sleep(&self, duration: Duration) {
        tokio::time::sleep(duration).await;
    }
}

/// Keeps calls at least `gap` apart; the first call goes straight through.
pub struct Limiter<C: Clock> {
    clock: C,
    gap: Duration,
    next_free: Mutex<Duration>,
}

impl<C: Clock> Limiter<C> {
    pub fn new(clock: C, gap: Duration) -> Self {
        Self {
            clock,
            gap,
            next_free: Mutex::new(Duration::ZERO),
        }
    }

    #[cfg(test)]
    pub fn clock(&self) -> &C {
        &self.clock
    }

    pub async fn wait(&self) {
        let now = self.clock.elapsed();
        // the slot is taken before sleeping, so concurrent callers queue up behind each other
        let wait = {
            let mut next_free = self
                .next_free
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let slot = (*next_free).max(now);
            *next_free = slot + self.gap;
            slot - now
        };
        if !wait.is_zero() {
            self.clock.sleep(wait).await;
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};

    /// Sleeping moves the clock forward and is recorded; nothing really waits.
    #[derive(Default)]
    pub struct FakeClock {
        now: Cell<Duration>,
        pub sleeps: RefCell<Vec<Duration>>,
    }

    impl FakeClock {
        pub fn advance(&self, by: Duration) {
            self.now.set(self.now.get() + by);
        }
    }

    impl Clock for FakeClock {
        fn elapsed(&self) -> Duration {
            self.now.get()
        }

        async fn sleep(&self, duration: Duration) {
            self.sleeps.borrow_mut().push(duration);
            self.advance(duration);
        }
    }

    const GAP: Duration = Duration::from_millis(1100);

    #[tokio::test]
    async fn the_first_call_does_not_wait() {
        let limiter = Limiter::new(FakeClock::default(), GAP);

        limiter.wait().await;

        assert!(limiter.clock.sleeps.borrow().is_empty());
    }

    #[tokio::test]
    async fn quick_calls_are_spaced_apart() {
        let limiter = Limiter::new(FakeClock::default(), GAP);

        limiter.wait().await;
        limiter.clock.advance(Duration::from_millis(100));
        limiter.wait().await;
        limiter.wait().await;

        assert_eq!(
            *limiter.clock.sleeps.borrow(),
            [GAP - Duration::from_millis(100), GAP]
        );
    }

    #[tokio::test]
    async fn a_call_after_a_long_pause_does_not_wait() {
        let limiter = Limiter::new(FakeClock::default(), GAP);

        limiter.wait().await;
        limiter.clock.advance(Duration::from_secs(5));
        limiter.wait().await;

        assert!(limiter.clock.sleeps.borrow().is_empty());
    }
}
