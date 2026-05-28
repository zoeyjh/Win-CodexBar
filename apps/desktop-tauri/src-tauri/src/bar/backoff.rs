use std::time::Duration;

use rand::Rng;

#[derive(Debug, Clone)]
pub struct BackoffPolicy {
    base: Duration,
    max: Duration,
    factor: f64,
    jitter_pct: f64,
    current_attempt: u32,
}

impl BackoffPolicy {
    pub fn new(base: Duration, max: Duration, factor: f64, jitter_pct: f64) -> Self {
        Self {
            base,
            max,
            factor,
            jitter_pct,
            current_attempt: 0,
        }
    }

    pub fn next_delay(&mut self) -> Duration {
        let delay_secs = self.base.as_secs_f64() * self.factor.powi(self.current_attempt as i32);
        let capped = delay_secs.min(self.max.as_secs_f64());
        let mut rng = rand::thread_rng();
        let jitter = rng.gen_range(-self.jitter_pct..=self.jitter_pct);
        let final_secs = capped * (1.0 + jitter);
        self.current_attempt += 1;
        Duration::from_secs_f64(final_secs.max(0.0))
    }

    pub fn reset(&mut self) {
        self.current_attempt = 0;
    }
}

pub fn rate_limit_backoff() -> BackoffPolicy {
    BackoffPolicy::new(
        Duration::from_secs(60),
        Duration::from_secs(30 * 60),
        2.0,
        0.2,
    )
}

pub fn server_error_backoff(base_poll_interval: Duration) -> BackoffPolicy {
    BackoffPolicy::new(base_poll_interval, Duration::from_secs(10 * 60), 2.0, 0.2)
}
