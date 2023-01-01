use std::time::Duration;

use google_drive3::hyper::{self, http};

pub struct UploadDelegateConfig {
    pub chunk_size: u64,
    pub backoff: Backoff,
}

pub struct UploadDelegate {
    config: UploadDelegateConfig,
    resumable_upload_url: Option<String>,
}

impl UploadDelegate {
    pub fn new(config: UploadDelegateConfig) -> UploadDelegate {
        UploadDelegate {
            config,
            resumable_upload_url: None,
        }
    }
}

impl google_drive3::client::Delegate for UploadDelegate {
    fn chunk_size(&mut self) -> u64 {
        self.config.chunk_size
    }

    fn cancel_chunk_upload(&mut self, _chunk: &google_drive3::client::ContentRange) -> bool {
        false
    }

    fn store_upload_url(&mut self, url: Option<&str>) {
        self.resumable_upload_url = url.map(|s| s.to_string())
    }

    fn upload_url(&mut self) -> Option<String> {
        self.resumable_upload_url.clone()
    }

    fn http_error(&mut self, _err: &hyper::Error) -> google_drive3::client::Retry {
        self.config.backoff.retry()
    }

    fn http_failure(
        &mut self,
        res: &http::response::Response<hyper::body::Body>,
        _err: Option<serde_json::Value>,
    ) -> google_drive3::client::Retry {
        if should_retry(res.status()) {
            self.config.backoff.retry()
        } else {
            self.config.backoff.abort()
        }
    }
}

fn should_retry(status: http::StatusCode) -> bool {
    status.is_server_error() || status == http::StatusCode::TOO_MANY_REQUESTS
}

pub struct RetryDelegateConfig {
    pub backoff: Backoff,
}

pub struct RetryDelegate {
    config: RetryDelegateConfig,
}

impl RetryDelegate {
    pub fn new(config: RetryDelegateConfig) -> RetryDelegate {
        RetryDelegate { config }
    }
}

impl google_drive3::client::Delegate for RetryDelegate {
    fn http_error(&mut self, _err: &hyper::Error) -> google_drive3::client::Retry {
        self.config.backoff.retry()
    }

    fn http_failure(
        &mut self,
        res: &http::response::Response<hyper::body::Body>,
        _err: Option<serde_json::Value>,
    ) -> google_drive3::client::Retry {
        if should_retry(res.status()) {
            self.config.backoff.retry()
        } else {
            self.config.backoff.abort()
        }
    }
}

pub struct BackoffConfig {
    pub max_retries: u32,
    pub min_sleep: Duration,
    pub max_sleep: Duration,
}

pub struct Backoff {
    attempts: u32,
    backoff: exponential_backoff::Backoff,
}

impl Backoff {
    pub fn new(config: BackoffConfig) -> Backoff {
        Backoff {
            attempts: 0,
            backoff: exponential_backoff::Backoff::new(
                config.max_retries,
                config.min_sleep,
                config.max_sleep,
            ),
        }
    }

    fn retry(&mut self) -> google_drive3::client::Retry {
        self.attempts += 1;
        self.backoff
            .next(self.attempts)
            .map(google_drive3::client::Retry::After)
            .unwrap_or(google_drive3::client::Retry::Abort)
    }

    fn abort(&mut self) -> google_drive3::client::Retry {
        google_drive3::client::Retry::Abort
    }
}
