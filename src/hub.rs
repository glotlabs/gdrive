use crate::config;
use google_drive3::hyper;
use google_drive3::hyper::client::HttpConnector;
use google_drive3::hyper_rustls::HttpsConnector;
use google_drive3::hyper_rustls::HttpsConnectorBuilder;
use google_drive3::oauth2;
use google_drive3::oauth2::authenticator::Authenticator;
use google_drive3::DriveHub;
use std::io;
use std::ops::Deref;
use std::path::PathBuf;

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

impl Auth {
    pub async fn new(secret: oauth2::ApplicationSecret) -> Result<Auth, io::Error> {
        let auth = oauth2::InstalledFlowAuthenticator::builder(
            secret,
            oauth2::InstalledFlowReturnMethod::Interactive,
        )
        .build()
        .await?;

        Ok(Auth(auth))
    }
}

fn prepare_secret(config: config::Secret) -> oauth2::ApplicationSecret {
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
