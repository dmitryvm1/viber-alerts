#![allow(unused_variables)]
#![feature(custom_attribute)]
#![feature(try_trait)]
#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
extern crate actix_web;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate futures;
extern crate json;
extern crate openssl;
extern crate r2d2;
#[macro_use]
extern crate diesel;
extern crate victoria_dom;
#[macro_use]
extern crate log;
extern crate actix;
extern crate chrono;
extern crate dirs;
extern crate env_logger;
extern crate forecast;
extern crate oauth2;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate tera;
extern crate url;
extern crate reqwest;
extern crate tokio_openssl;

use url::Url;
use oauth2::basic::BasicClient;
use oauth2::prelude::*;
use oauth2::{AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
             TokenUrl};
use actix_web::middleware::identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{http, middleware, App};
use std::sync::Arc;
use weather::*;
// use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use actix::{Actor, Arbiter, AsyncContext, Context, Running};
use actix_web::server::HttpServer;
use actix_web::*;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use futures::{Future, Stream};
use std::env;
use std::sync::Mutex;
use std::sync::RwLock;
use actix::Handler;
use common::messages::*;
use actix::Recipient;
use actix::Message;
use viber::messages;

static APP_NAME: &str = "viber_alerts";

pub mod api;
pub mod bitcoin;
pub mod common;
pub mod config;
pub mod models;
pub mod scheduler;
pub mod schema;
pub mod viber;
pub mod weather;

// Interval between the task executions where all the notification/alert logic happens.
#[cfg(debug_assertions)]
static QUERY_INTERVAL: u64 = 6;
#[cfg(not(debug_assertions))]
static QUERY_INTERVAL: u64 = 60;

pub type AppStateType = Arc<RwLock<AppState>>;
type PgPool = Pool<ConnectionManager<PgConnection>>;

impl Actor for WeatherInquirer {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        {
            let mut state = self.app_state.write().unwrap();
            if  self.viber
                .update_subscribers(&mut state.subscribers)
                .is_err()
            {
                warn!("Failed to read subscribers.");
            };
        }
        ctx.run_interval(
            std::time::Duration::new(QUERY_INTERVAL, 0),
            |_t: &mut WeatherInquirer, _ctx: &mut Context<Self>| {
                match _t.inquire_if_needed() {
                    Err(e) => {
                        error!("Error inquiring weather forecast. {}", e.as_fail());
                    }
                    Ok(success) => {
                        if success {
                            let mut state = _t.app_state.write().unwrap();
                            _t
                                .viber
                                .update_subscribers(&mut state.subscribers)
                                .map_err(|e| {
                                    warn!("Failed to read subscribers. {:?}", e);
                                });
                        }
                    }
                };
                _t.try_broadcast();
            },
        );
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        Running::Stop
    }
}

impl Handler<WorkerUnit> for WeatherInquirer {
    type Result = ();

    fn handle(&mut self, msg: WorkerUnit, ctx: & mut Context<Self>) -> Self::Result {
        match msg {
            WorkerUnit::BTCPrice { user_id } => {
                self.send_btc_price(&user_id);
            },
            WorkerUnit::TomorrowForecast { user_id } => {
                debug!("handling tomorrow forecast");
                self.send_forecast_for_tomorrow(&user_id).map_err(|_| {
                    error!("Can't send forecast for tomorrow to {}", &user_id);
                });
            },
            _ => { }
        };
        ()
    }
}

pub struct AppState {
    pub addr: Mutex<Option<Recipient<WorkerUnit>>>,
    pub config: config::Config,
    pub subscribers: Vec<messages::Member>,
    pub last_text_broadcast: scheduler::TryTillSuccess,
    pub last_btc_update: scheduler::TryTillSuccess,
    pub pool: PgPool,
    pub auth_client: Option<BasicClient>,
    template: tera::Tera, // <- store tera template in application state
}

impl AppState {
    pub fn new(config: &config::Config, pool: PgPool) -> AppState {
        let viber_api_key = config.viber_api_key.clone();
        let admin_id = config.admin_id.clone();

        let tera = tera::Tera::new("templates/**/*").expect("Failed to load templates");
        AppState {
            config: (*config).clone(),
            subscribers: Vec::new(),
            last_text_broadcast: scheduler::TryTillSuccess::new(),
            last_btc_update: scheduler::TryTillSuccess::new(),
            template: tera,
            pool,
            auth_client: None,
            addr: Mutex::new(None)
        }
    }
}

fn get_server_port() -> u16 {
    env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080)
}

fn prepare_google_auth(config: &config::Config) -> BasicClient {
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
fn main() {
    use std::borrow::BorrowMut;

    env::set_var("RUST_LOG", "viber_alerts=debug");
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();
    let sys = actix::System::new(APP_NAME);

    let mut privkey_path = config::Config::get_config_dir(APP_NAME);
    let mut fullchain_path = privkey_path.clone();
    privkey_path.push("privkey.pem");
    fullchain_path.push("fullchain.pem");

    // load ssl keys
    // let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    // builder
    //      .set_private_key_file(privkey_path.to_str().unwrap(), SslFiletype::PEM)
    //       .unwrap();
    //   builder.set_certificate_chain_file(fullchain_path.to_str().unwrap()).unwrap();

    let config = config::Config::read(APP_NAME);
    info!("Connecting to the database:");
    let db_url = config.database_url.clone().expect("No db url.");
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");
    let mut state = Arc::new(RwLock::new(AppState::new(&config, pool)));
    let _state = state.clone();
    state.write().unwrap().auth_client = Some(prepare_google_auth(&config));
    let _server = Arbiter::start(move |ctx: &mut Context<_>| weather::WeatherInquirer::new(_state));
    let forecast_addr = _server.recipient();
    {
        state.write().unwrap().addr = Mutex::new(Some(forecast_addr));
    }


    let addr = HttpServer::new(move || {
        App::with_state(state.clone())
            .middleware(middleware::Logger::default())
            .middleware(IdentityService::new(
                CookieIdentityPolicy::new(&[0; 32])
                    .name("auth-example")
                    .secure(false),
            ))
            .handler("/api/static", fs::StaticFiles::new("static/").unwrap())
            .resource("/login", |r| r.method(http::Method::POST).with(api::login))
            .resource("/logout", |r| r.f(api::logout))
            .resource("/api/google_oauth/", |r| r.f(api::google_oauth))
            .resource("/", |r| r.f(api::index))
            .resource("/google6e03bff5229f1e21.html", |r| r.f(|_| "google-site-verification: google6e03bff5229f1e21.html"))
            .resource("/users", |r| r.f(api::users))
            .resource("/list", |r| r.method(http::Method::GET).with(api::list))
            .resource("/api/send_message/", |r| r.f(api::send_message))
            .resource("/api/acc_data/", |r| r.f(api::acc_data))
            .resource("/api/viber/webhook/", |r| r.f(api::viber_webhook))
    })
    .bind(format!("0.0.0.0:{}", get_server_port()))
    .unwrap()
    .workers(1)
    .shutdown_timeout(1)
    .start();

    let _ = sys.run();
}
