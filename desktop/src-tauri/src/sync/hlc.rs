use crate::error::AppResult;
use sqlx::SqlitePool;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

// low bits count events inside the same millisecond
const COUNTER_BITS: u32 = 16;

/// Hybrid logical clock: unix ms shifted left by 16 bits, so values stay
/// ordered by time and never repeat.
#[derive(Debug, Default)]
pub struct Hlc {
    last: AtomicU64,
}

impl Hlc {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn now(&self) -> u64 {
        self.advance(wall_clock() << COUNTER_BITS)
    }

    /// Call with every remote value so local values always sort after it.
    pub fn merge(&self, remote: u64) -> u64 {
        self.advance((wall_clock() << COUNTER_BITS).max(remote.saturating_add(1)))
    }

    // hlc must never go backwards, even if the system clock does
    fn advance(&self, floor: u64) -> u64 {
        let mut last = self.last.load(Ordering::Acquire);
        loop {
            let next = last.saturating_add(1).max(floor);
            match self
                .last
                .compare_exchange_weak(last, next, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => return next,
                Err(current) => last = current,
            }
        }
    }
}

/// Moves the clock past every stored change, so a restart cannot hand out older values.
pub async fn restore(pool: &SqlitePool, hlc: &Hlc) -> AppResult<()> {
    let newest: Option<i64> = sqlx::query_scalar("SELECT MAX(hlc) FROM change_log")
        .fetch_one(pool)
        .await?;
    if let Some(newest) = newest.and_then(|v| u64::try_from(v).ok()) {
        hlc.merge(newest);
    }
    Ok(())
}

fn wall_clock() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn increases_over_many_calls() {
        let hlc = Hlc::new();
        let mut previous = hlc.now();
        for _ in 0..100_000 {
            let next = hlc.now();
            assert!(next > previous);
            previous = next;
        }
    }

    #[test]
    fn keeps_increasing_when_clock_goes_backwards() {
        let hlc = Hlc::new();
        let ahead = hlc.advance(u64::MAX / 2);
        // a floor far below the last value is what a clock jump back looks like
        assert!(hlc.advance(1) > ahead);
        assert!(hlc.now() > ahead);
    }

    #[test]
    fn merge_sorts_after_remote_value() {
        let hlc = Hlc::new();
        let remote = hlc.now() + (5_000 << COUNTER_BITS);
        assert!(hlc.merge(remote) > remote);
        assert!(hlc.now() > remote);
    }

    #[test]
    fn merge_ignores_older_remote_value() {
        let hlc = Hlc::new();
        let local = hlc.now();
        assert!(hlc.merge(1) > local);
    }
}
