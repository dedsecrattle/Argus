use std::time::Duration;

use tokio::time::sleep;

use crate::error::FetchError;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

impl RetryConfig {
    pub fn new(max_retries: u32) -> Self {
        Self {
            max_retries,
            ..Default::default()
        }
    }

    pub fn with_initial_backoff(mut self, duration: Duration) -> Self {
        self.initial_backoff = duration;
        self
    }

    pub fn with_max_backoff(mut self, duration: Duration) -> Self {
        self.max_backoff = duration;
        self
    }

    pub fn with_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    pub fn calculate_backoff(&self, attempt: u32) -> Duration {
        let backoff_ms = self.initial_backoff.as_millis() as f64
            * self.backoff_multiplier.powi(attempt as i32);
        let backoff = Duration::from_millis(backoff_ms as u64);
        backoff.min(self.max_backoff)
    }
}

pub struct RetryState {
    config: RetryConfig,
    attempt: u32,
}

impl RetryState {
    pub fn new(config: RetryConfig) -> Self {
        Self { config, attempt: 0 }
    }

    pub async fn retry<F, T, E>(&mut self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> Result<T, E>,
        E: Into<FetchError> + From<FetchError>,
    {
        loop {
            match operation() {
                Ok(result) => return Ok(result),
                Err(err) => {
                    let fetch_err: FetchError = err.into();

                    if !fetch_err.is_retryable() {
                        return Err(fetch_err.into());
                    }

                    if self.attempt >= self.config.max_retries {
                        tracing::warn!(
                            "max retries ({}) reached: {}",
                            self.config.max_retries,
                            fetch_err
                        );
                        return Err(fetch_err.into());
                    }

                    let backoff = self.config.calculate_backoff(self.attempt);
                    tracing::debug!(
                        "retry attempt {} after {:?}: {}",
                        self.attempt + 1,
                        backoff,
                        fetch_err
                    );

                    sleep(backoff).await;
                    self.attempt += 1;
                }
            }
        }
    }

    pub fn attempt(&self) -> u32 {
        self.attempt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_retry_config() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_backoff, Duration::from_millis(100));
        assert_eq!(config.max_backoff, Duration::from_secs(30));
        assert_eq!(config.backoff_multiplier, 2.0);
    }

    #[test]
    fn calculate_exponential_backoff() {
        let config = RetryConfig::default();
        assert_eq!(config.calculate_backoff(0), Duration::from_millis(100));
        assert_eq!(config.calculate_backoff(1), Duration::from_millis(200));
        assert_eq!(config.calculate_backoff(2), Duration::from_millis(400));
        assert_eq!(config.calculate_backoff(3), Duration::from_millis(800));
    }

    #[test]
    fn backoff_respects_max() {
        let config = RetryConfig::default().with_max_backoff(Duration::from_millis(300));
        assert_eq!(config.calculate_backoff(0), Duration::from_millis(100));
        assert_eq!(config.calculate_backoff(1), Duration::from_millis(200));
        assert_eq!(config.calculate_backoff(2), Duration::from_millis(300));
        assert_eq!(config.calculate_backoff(3), Duration::from_millis(300));
    }

    #[test]
    fn custom_multiplier() {
        let config = RetryConfig::default().with_multiplier(3.0);
        assert_eq!(config.calculate_backoff(0), Duration::from_millis(100));
        assert_eq!(config.calculate_backoff(1), Duration::from_millis(300));
        assert_eq!(config.calculate_backoff(2), Duration::from_millis(900));
    }

    #[tokio::test]
    async fn retry_succeeds_on_second_attempt() {
        let config = RetryConfig::new(3);
        let mut state = RetryState::new(config);
        let mut attempts = 0;

        let result = state
            .retry(|| {
                attempts += 1;
                if attempts < 2 {
                    Err(FetchError::new(
                        crate::error::FetchErrorKind::Timeout,
                        "timeout".to_string(),
                    ))
                } else {
                    Ok(42)
                }
            })
            .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts, 2);
    }

    #[tokio::test]
    async fn retry_fails_after_max_attempts() {
        let config = RetryConfig::new(2);
        let mut state = RetryState::new(config);
        let mut attempts = 0;

        let result: Result<i32, FetchError> = state
            .retry(|| {
                attempts += 1;
                Err(FetchError::new(
                    crate::error::FetchErrorKind::Timeout,
                    "timeout".to_string(),
                ))
            })
            .await;

        assert!(result.is_err());
        assert_eq!(attempts, 3);
    }

    #[tokio::test]
    async fn non_retryable_error_fails_immediately() {
        let config = RetryConfig::new(3);
        let mut state = RetryState::new(config);
        let mut attempts = 0;

        let result: Result<i32, FetchError> = state
            .retry(|| {
                attempts += 1;
                Err(FetchError::new(
                    crate::error::FetchErrorKind::ClientError,
                    "404".to_string(),
                ))
            })
            .await;

        assert!(result.is_err());
        assert_eq!(attempts, 1);
    }
}
