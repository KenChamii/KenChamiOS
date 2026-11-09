use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const MAX_FAILURES: usize = 10;
const WINDOW: Duration = Duration::from_secs(60);
const BLOCK: Duration = Duration::from_secs(60);

#[derive(Default)]
struct Record {
    failures: Vec<Instant>,
    blocked_until: Option<Instant>,
}

/// Blocks an address for a minute after 10 failed authentications within a minute.
#[derive(Default)]
pub struct FailureLimiter {
    records: Mutex<HashMap<IpAddr, Record>>,
}

impl FailureLimiter {
    pub fn is_blocked(&self, ip: IpAddr, now: Instant) -> bool {
        let Ok(records) = self.records.lock() else {
            return true;
        };
        records
            .get(&ip)
            .and_then(|r| r.blocked_until)
            .is_some_and(|until| until > now)
    }

    pub fn record_failure(&self, ip: IpAddr, now: Instant) {
        let Ok(mut records) = self.records.lock() else {
            return;
        };
        records.retain(|_, r| {
            r.blocked_until.is_some_and(|until| until > now)
                || r.failures.iter().any(|t| now.duration_since(*t) < WINDOW)
        });
        let record = records.entry(ip).or_default();
        record.failures.retain(|t| now.duration_since(*t) < WINDOW);
        record.failures.push(now);
        if record.failures.len() >= MAX_FAILURES {
            record.blocked_until = Some(now + BLOCK);
            record.failures.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(last: u8) -> IpAddr {
        IpAddr::from([192, 168, 1, last])
    }

    #[test]
    fn blocks_after_ten_failures_and_unblocks_later() {
        let limiter = FailureLimiter::default();
        let start = Instant::now();
        for _ in 0..9 {
            limiter.record_failure(ip(5), start);
        }
        assert!(!limiter.is_blocked(ip(5), start));

        limiter.record_failure(ip(5), start);
        assert!(limiter.is_blocked(ip(5), start));
        assert!(!limiter.is_blocked(ip(6), start));
        assert!(!limiter.is_blocked(ip(5), start + BLOCK + Duration::from_secs(1)));
    }

    #[test]
    fn old_failures_do_not_add_up() {
        let limiter = FailureLimiter::default();
        let start = Instant::now();
        for _ in 0..9 {
            limiter.record_failure(ip(5), start);
        }
        let later = start + WINDOW + Duration::from_secs(1);
        limiter.record_failure(ip(5), later);
        assert!(!limiter.is_blocked(ip(5), later));
    }
}
