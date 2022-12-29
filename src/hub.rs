use crate::config;
use google_drive3::hyper;
use google_drive3::hyper::client::HttpConnector;
use google_drive3::hyper_rustls::HttpsConnector;
use google_drive3::hyper_rustls::HttpsConnectorBuilder;
use google_drive3::oauth2;
use google_drive3::oauth2::authenticator::Authenticator;
use google_drive3::oauth2::authenticator_delegate::InstalledFlowDelegate;
use google_drive3::DriveHub;
use std::future::Future;
use std::io;
use std::ops::Deref;
use std::path::PathBuf;
use std::pin::Pin;

pub struct HubConfig {
    pub secret: oauth2::ApplicationSecret,
    pub tokens_path: PathBuf,
}

pub struct Hub(DriveHub<HttpsConnector<HttpConnector>>);

impl Deref for Hub {
    type Target = DriveHub<HttpsConnector<HttpConnector>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Hub {
    pub async fn new(auth: Auth) -> Hub {
        let connector = HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .build();

        let http_client = hyper::Client::builder().build(connector);

        Hub(google_drive3::DriveHub::new(http_client, auth.0))
    }
}

pub struct Auth(pub Authenticator<HttpsConnector<HttpConnector>>);

impl Deref for Auth {
    type Target = Authenticator<HttpsConnector<HttpConnector>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Auth {
    pub async fn new(config: config::Secret, tokens_path: &PathBuf) -> Result<Auth, io::Error> {
        let secret = oauth2_secret(config);
        let delegate = Box::new(AuthDelegate);

        let auth = oauth2::InstalledFlowAuthenticator::builder(
            secret,
            oauth2::InstalledFlowReturnMethod::HTTPPortRedirect(8085),
        )
        .persist_tokens_to_disk(tokens_path)
        .flow_delegate(delegate)
        .build()
        .await?;

        Ok(Auth(auth))
    }
}

fn oauth2_secret(config: config::Secret) -> oauth2::ApplicationSecret {
    oauth2::ApplicationSecret {
        client_id: config.client_id,
        client_secret: config.client_secret,
        token_uri: "https://oauth2.googleapis.com/token".to_string(),
        auth_uri: "https://accounts.google.com/o/oauth2/auth".to_string(),
        redirect_uris: vec!["urn:ietf:wg:oauth:2.0:oob".to_string()],
        project_id: None,
        client_email: None,
        auth_provider_x509_cert_url: Some("https://www.googleapis.com/oauth2/v1/certs".to_string()),
        client_x509_cert_url: None,
    }
}

struct AuthDelegate;

impl InstalledFlowDelegate for AuthDelegate {
    fn present_user_url<'a>(
        &'a self,
        url: &'a str,
        _need_code: bool,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + 'a>> {
        Box::pin(present_user_url(url))
    }
}

async fn present_user_url(url: &str) -> Result<String, String> {
    println!();
    println!("Open the url in your browser and follow the instructions displayed there:");
    println!("{}", url);
    Ok(String::new())
}
