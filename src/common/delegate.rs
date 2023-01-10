use google_drive3::hyper;
use google_drive3::hyper::http;
use human_bytes::human_bytes;
use std::time::Duration;

pub struct UploadDelegateConfig {
    pub chunk_size: u64,
    pub backoff: Backoff,
    pub print_chunk_errors: bool,
    pub print_chunk_info: bool,
}

pub struct UploadDelegate {
    config: UploadDelegateConfig,
    resumable_upload_url: Option<String>,
    previous_chunk: Option<google_drive3::client::ContentRange>,
}

impl UploadDelegate {
    pub fn new(config: UploadDelegateConfig) -> UploadDelegate {
        UploadDelegate {
            config,
            resumable_upload_url: None,
            previous_chunk: None,
        }
    }

    fn print_chunk_info(&self, chunk: &google_drive3::client::ContentRange) {
        if self.config.print_chunk_info {
            if let Some(range) = &chunk.range {
                let chunk_size = range.last - range.first + 1;

                let action = if Some(chunk) == self.previous_chunk.as_ref() {
                    "Retrying"
                } else {
                    "Uploading"
                };

                println!(
                    "Info: {} {} chunk ({}-{} of {})",
                    action,
                    human_bytes(chunk_size as f64),
                    range.first,
                    range.last,
                    chunk.total_length
                )
            }
        }
    }
}

impl google_drive3::client::Delegate for UploadDelegate {
    fn chunk_size(&mut self) -> u64 {
        self.config.chunk_size
    }

    fn cancel_chunk_upload(&mut self, chunk: &google_drive3::client::ContentRange) -> bool {
        self.print_chunk_info(chunk);
        self.previous_chunk = Some(chunk.clone());

        false
    }

    fn store_upload_url(&mut self, url: Option<&str>) {
        self.resumable_upload_url = url.map(|s| s.to_string())
    }

    fn upload_url(&mut self) -> Option<String> {
        self.resumable_upload_url.clone()
    }

    fn http_error(&mut self, err: &hyper::Error) -> google_drive3::client::Retry {
        if self.config.print_chunk_errors {
            eprintln!("Warning: Failed attempt to upload chunk: {}", err);
        }
        self.config.backoff.retry()
    }

    fn http_failure(
        &mut self,
        res: &http::response::Response<hyper::body::Body>,
        _err: Option<serde_json::Value>,
    ) -> google_drive3::client::Retry {
        let status = res.status();

        if should_retry(status) {
            if self.config.print_chunk_errors {
                eprintln!(
                    "Warning: Failed attempt to upload chunk. Status code: {}, body: {:?}",
                    status,
                    res.body()
                );
            }
            self.config.backoff.retry()
        } else {
            self.config.backoff.abort()
        }
    }
}

fn should_retry(status: http::StatusCode) -> bool {
    status.is_server_error() || status == http::StatusCode::TOO_MANY_REQUESTS
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
