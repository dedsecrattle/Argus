use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum FetchErrorKind {
    Timeout,
    ConnectionRefused,
    DnsResolution,
    TooManyRedirects,
    TlsError,
    RateLimited,
    ServerError,
    ClientError,
    NetworkError,
    InvalidUrl,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct FetchError {
    pub kind: FetchErrorKind,
    pub message: String,
    pub status_code: Option<u16>,
    pub retryable: bool,
}

impl FetchError {
    pub fn new(kind: FetchErrorKind, message: String) -> Self {
        let retryable = matches!(
            kind,
            FetchErrorKind::Timeout
                | FetchErrorKind::ConnectionRefused
                | FetchErrorKind::DnsResolution
                | FetchErrorKind::RateLimited
                | FetchErrorKind::ServerError
                | FetchErrorKind::NetworkError
        );

        Self {
            kind,
            message,
            status_code: None,
            retryable,
        }
    }

    pub fn with_status(mut self, status: u16) -> Self {
        self.status_code = Some(status);
        self
    }

    pub fn from_reqwest(err: &reqwest::Error) -> Self {
        if err.is_timeout() {
            return Self::new(FetchErrorKind::Timeout, err.to_string());
        }

        if err.is_connect() {
            return Self::new(FetchErrorKind::ConnectionRefused, err.to_string());
        }

        if err.is_redirect() {
            return Self::new(FetchErrorKind::TooManyRedirects, err.to_string());
        }

        if let Some(status) = err.status() {
            let code = status.as_u16();
            if code == 429 {
                return Self::new(FetchErrorKind::RateLimited, err.to_string())
                    .with_status(code);
            }
            if (500..600).contains(&code) {
                return Self::new(FetchErrorKind::ServerError, err.to_string())
                    .with_status(code);
            }
            if (400..500).contains(&code) {
                return Self::new(FetchErrorKind::ClientError, err.to_string())
                    .with_status(code);
            }
        }

        Self::new(FetchErrorKind::NetworkError, err.to_string())
    }

    pub fn is_retryable(&self) -> bool {
        self.retryable
    }
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)?;
        if let Some(status) = self.status_code {
            write!(f, " (status: {})", status)?;
        }
        Ok(())
    }
}

impl std::error::Error for FetchError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeout_is_retryable() {
        let err = FetchError::new(FetchErrorKind::Timeout, "timeout".to_string());
        assert!(err.is_retryable());
    }

    #[test]
    fn client_error_not_retryable() {
        let err = FetchError::new(FetchErrorKind::ClientError, "404".to_string());
        assert!(!err.is_retryable());
    }

    #[test]
    fn server_error_is_retryable() {
        let err = FetchError::new(FetchErrorKind::ServerError, "500".to_string());
        assert!(err.is_retryable());
    }

    #[test]
    fn rate_limited_is_retryable() {
        let err = FetchError::new(FetchErrorKind::RateLimited, "429".to_string());
        assert!(err.is_retryable());
    }
}
