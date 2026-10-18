use std::time::{Duration, SystemTime};

pub const MIN_X: Duration = Duration::from_secs(3600);
pub const MAX_X: Duration = Duration::from_secs(86400);
pub const DEFAULT_X: Duration = Duration::from_secs(6 * 3600);
const WEEK: u64 = 7 * 86400;
const MIN_LEAD: Duration = Duration::from_secs(5 * 60);

/// Spread all postcards still to be published (scheduled + about to be
/// scheduled) over a week, within [1h, 1d].
pub fn interval_for(pending: usize) -> Duration {
    if pending == 0 {
        return DEFAULT_X;
    }
    Duration::from_secs_f64(WEEK as f64 / pending as f64).clamp(MIN_X, MAX_X)
}

/// None means "publish right now".
pub fn next_time(now: SystemTime, last: Option<SystemTime>, x: Duration) -> Option<SystemTime> {
    let t = last? + x.clamp(MIN_X, MAX_X);
    (t > now + MIN_LEAD).then_some(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interval() {
        assert_eq!(interval_for(1), MAX_X);
        assert_eq!(interval_for(1000), MIN_X);
        assert_eq!(interval_for(14), Duration::from_secs(12 * 3600));
        assert_eq!(interval_for(0), DEFAULT_X);
    }

    #[test]
    fn next() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
        let h = Duration::from_secs(3600);
        assert_eq!(next_time(now, None, h), None);
        assert_eq!(next_time(now, Some(now - 10 * h), h), None);
        assert_eq!(next_time(now, Some(now + h), 6 * h), Some(now + 7 * h));
        assert_eq!(
            next_time(now, Some(now + h), 100 * h),
            Some(now + h + MAX_X)
        );
    }
}
