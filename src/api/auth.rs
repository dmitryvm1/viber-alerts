use oauth2::basic::BasicClient;
use oauth2::prelude::*;
use oauth2::{AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
             TokenUrl};
use config;
use url::Url;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GoogleProfile {
    pub id: Option<String>,
    pub email: Option<String>,
    pub verified_email: Option<bool>,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub link: Option<String>,
    pub picture: Option<String>,
    pub gender: Option<String>,
    pub locale: Option<String>,
}


pub fn prepare_google_auth(config: &config::Config) -> BasicClient {
    let google_client_id = ClientId::new(
        config.google_client_id.clone().unwrap()
    );
    let google_client_secret = ClientSecret::new(
        config.google_client_secret.clone().unwrap()
    );
    let auth_url = AuthUrl::new(
        Url::parse("https://accounts.google.com/o/oauth2/v2/auth")
            .expect("Invalid authorization endpoint URL"),
    );
    let token_url = TokenUrl::new(
        Url::parse("https://www.googleapis.com/oauth2/v4/token")
            .expect("Invalid token endpoint URL"),
    );

    // Set up the config for the Google OAuth2 process.
    let client = BasicClient::new(
        google_client_id,
        Some(google_client_secret),
        auth_url,
        Some(token_url)
    )
        .add_scope(Scope::new("https://www.googleapis.com/auth/userinfo.profile".to_owned()))
        .add_scope(Scope::new("https://www.googleapis.com/auth/userinfo.email".to_owned()))
        .add_scope(Scope::new("https://www.googleapis.com/auth/plus.me".to_owned()))
        .set_redirect_url(
            RedirectUrl::new(
                Url::parse(&format!("{}api/google_oauth/", &config.domain_root_url.clone().unwrap()))
                    .expect("Invalid redirect URL")
            )
        );
    client
}