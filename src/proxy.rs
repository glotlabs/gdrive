/// modified from https://github.com/algesten/ureq/

use http::Uri;
use std::io;

/// Proxy protocol
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub(crate) enum Proto {
    Http,
    Https,
}

/// Proxy server settings
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct EnvProxy {
    proxy: String,
}

impl EnvProxy {
    /// Create a proxy from a uri.
    ///
    /// # Arguments:
    ///
    /// * `proxy` - a str of format `<protocol>://<user>:<password>@<host>:port` . All parts
    ///    except host are optional.
    ///
    /// ###  Protocols
    ///
    /// * `http`: HTTP CONNECT proxy
    /// * `https`: HTTPS CONNECT proxy (requires a TLS provider)
    ///
    /// # Examples proxy formats
    ///
    /// * `http://127.0.0.1:8080`
    /// * `localhost`

    fn new_with_flag(proxy: &str) -> Result<Self, Error> {
        let uri = proxy.parse::<Uri>().unwrap();

        // The uri must have an authority part (with the host), or
        // it is invalid.
        let _ = uri.authority().ok_or(Error::InvalidProxyUrl)?;

        // The default protocol is Proto::HTTP
        let scheme = uri.scheme_str().unwrap_or("http");
        let _proto = match scheme {
            "http" => Proto::Http,
            "https" => Proto::Https,
            &_ => todo!("only support http or https protocol!"),
        };

        Ok(Self {
            proxy: proxy.to_string(),
        })
    }

    /// Read proxy settings from environment variables.
    ///
    /// The environment variable is expected to contain a proxy URI. The following
    /// environment variables are attempted:
    ///
    /// * `ALL_PROXY`
    /// * `HTTPS_PROXY`
    /// * `HTTP_PROXY`
    ///
    /// Returns `None` if no environment variable is set or the URI is invalid.
    pub fn try_from_env() -> Option<Self> {
        macro_rules! try_env {
            ($($env:literal),+) => {
                $(
                    if let Ok(env) = std::env::var($env) {
                        if let Ok(proxy) = Self::new_with_flag(&env) {
                            return Some(proxy);
                        }
                    }
                )+
            };
        }

        try_env!(
            "ALL_PROXY",
            "all_proxy",
            "HTTPS_PROXY",
            "https_proxy",
            "HTTP_PROXY",
            "http_proxy"
        );
        None
    }
    
    pub fn uri_str(&self) -> &String {
        &self.proxy
    }
    
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {

    /// When [`http_status_as_error()`](crate::config::ConfigBuilder::http_status_as_error) is true,
    /// 4xx and 5xx response status codes are translated to this error.
    ///
    /// This is the default behavior.
    StatusCode(u16),

    /// Errors arising from the http-crate.
    ///
    /// These errors happen for things like invalid characters in header names.
    Http,

    /// Error if the URI is missing scheme or host.
    BadUri(String),

    /// Error in io such as the TCP socket.
    Io(io::Error),

    /// Error when resolving a hostname fails.
    HostNotFound,

    /// A redirect failed.
    ///
    /// This happens when ureq encounters a redirect when sending a request body
    /// such as a POST request, and receives a 307/308 response. ureq refuses to
    /// redirect the POST body and instead raises this error.
    RedirectFailed,

    /// Error when creating proxy settings.
    InvalidProxyUrl,

    /// A connection failed.
    ConnectionFailed,

    /// A send body (Such as `&str`) is larger than the `content-length` header.
    BodyExceedsLimit(u64),
}