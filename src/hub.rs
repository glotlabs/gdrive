use crate::app_config;
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
use hyper_proxy::{ProxyConnector, Intercept, Proxy};
use headers::Authorization;
use url::Url;
use hyper::Client;
use crate::proxy::EnvProxy;

pub struct HubConfig {
    pub secret: oauth2::ApplicationSecret,
    pub tokens_path: PathBuf,
}

pub struct Hub(DriveHub<ProxyConnector<HttpsConnector<HttpConnector>>>);

impl Deref for Hub {
    type Target = DriveHub<ProxyConnector<HttpsConnector<HttpConnector>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Hub {
    pub async fn new(auth: Auth) -> Hub {
        let http_client = create_client();
        Hub(google_drive3::DriveHub::new(http_client, auth.0))
    }
}

pub struct Auth(pub Authenticator<ProxyConnector<HttpsConnector<HttpConnector>>>);

impl Deref for Auth {
    type Target = Authenticator<ProxyConnector<HttpsConnector<HttpConnector>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Auth {
    pub async fn new(
        config: &app_config::Secret,
        tokens_path: &PathBuf,
    ) -> Result<Auth, io::Error> {
        let secret = oauth2_secret(config);
        let delegate = Box::new(AuthDelegate);
        let client = create_client();

        let auth = oauth2::InstalledFlowAuthenticator::with_client::<hyper::Client<ProxyConnector<HttpsConnector<HttpConnector>>>>(
            secret,
            oauth2::InstalledFlowReturnMethod::HTTPPortRedirect(8085),
            client,
        )
        .persist_tokens_to_disk(tokens_path)
        .flow_delegate(delegate)
        .build()
        .await?;

        Ok(Auth(auth))
    }
}

fn oauth2_secret(config: &app_config::Secret) -> oauth2::ApplicationSecret {
    oauth2::ApplicationSecret {
        client_id: config.client_id.clone(),
        client_secret: config.client_secret.clone(),
        token_uri: String::from("https://oauth2.googleapis.com/token"),
        auth_uri: String::from("https://accounts.google.com/o/oauth2/auth"),
        redirect_uris: vec![String::from("urn:ietf:wg:oauth:2.0:oob")],
        project_id: None,
        client_email: None,
        auth_provider_x509_cert_url: Some(String::from(
            "https://www.googleapis.com/oauth2/v1/certs",
        )),
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
    println!();
    println!("Gdrive requires permissions to manage your files on Google Drive.");
    println!("Open the url in your browser and follow the instructions:");
    println!("{}", url);
    Ok(String::new())
}

fn create_client() -> Client<ProxyConnector<HttpsConnector<HttpConnector>>> {
    let connector = HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_or_http()
        .enable_http1()
        .enable_http2()
        .build();
    let env_proxy = EnvProxy::try_from_env();
    let proxy_connector = match env_proxy {
        Some(val) => {
            let uri_str = val.uri_str();
            let mut url = Url::parse(uri_str).unwrap();
            let username = String::from(url.username()).clone();
            let password = String::from(url.password().unwrap_or_default()).clone();
            let _ = url.set_username("");
            let _ = url.set_password(None);
            let uri = url.as_str().parse();
            let mut proxy = Proxy::new(Intercept::All, uri.unwrap());
            if username != "" {
                proxy.set_authorization(Authorization::basic(&username, &password));            
            }
            println!("using system proxy {}", uri_str);
            ProxyConnector::from_proxy(connector, proxy).unwrap()
        },
        None => {
            // println!("not using proxy!");
            ProxyConnector::new(connector).unwrap()
        }
    };
    hyper::Client::builder().build(proxy_connector)
}