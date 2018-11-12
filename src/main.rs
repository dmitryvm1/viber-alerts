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
extern crate reqwest;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate tera;

use actix_web::middleware::identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{
    http, middleware, App
};
use std::sync::Arc;
use weather::*;
// use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use actix::{Actor, Arbiter, AsyncContext, Context, Running};
use actix_web::error;
use actix_web::server::HttpServer;
use actix_web::*;
use chrono::Datelike;
use chrono::TimeZone;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use futures::{Future, Stream};
use models::{NewPost, Post};
use std::collections::HashMap;
use std::env;
use std::ops::Deref;
use std::sync::Mutex;
use std::sync::RwLock;
use actix::Recipient;

static APP_NAME: &str = "viber_alerts";

pub mod config;
pub mod models;
pub mod scheduler;
pub mod schema;
pub mod viber;
pub mod weather;
pub mod api;

// Interval between the task executions where all the notification/alert logic happens.
#[cfg(debug_assertions)]
static QUERY_INTERVAL: u64 = 6;
#[cfg(not(debug_assertions))]
static QUERY_INTERVAL: u64 = 60;

pub type AppStateType = Arc<AppState>;
type PgPool = Pool<ConnectionManager<PgConnection>>;

impl Actor for WeatherInquirer {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        if self
            .app_state
            .viber
            .lock()
            .unwrap()
            .update_subscribers()
            .is_err()
        {
            warn!("Failed to read subscribers.");
        };
        ctx.run_interval(
            std::time::Duration::new(QUERY_INTERVAL, 0),
            |_t: &mut WeatherInquirer, _ctx: &mut Context<Self>| {
                match _t.inquire_if_needed() {
                    Err(e) => {
                        error!("Error inquiring weather forecast. {}", e.as_fail());
                    }
                    Ok(success) => {
                        if success {
                            _t.app_state
                                .viber
                                .lock()
                                .unwrap()
                                .update_subscribers()
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

pub struct AppState {
    pub config: config::Config,
    pub viber: Mutex<viber::Viber>,
    pub last_text_broadcast: RwLock<scheduler::TryTillSuccess>,
    pub pool: PgPool,
    template: tera::Tera, // <- store tera template in application state
}

impl AppState {
    pub fn new(config: config::Config, pool: PgPool) -> AppState {
        let viber_api_key = config.viber_api_key.clone();
        let admin_id = config.admin_id.clone();

        let tera = tera::Tera::new("templates/**/*").expect("Failed to load templates");
        AppState {
            config: config,
            viber: Mutex::new(viber::Viber::new(viber_api_key.unwrap(), admin_id.unwrap())),
            last_text_broadcast: RwLock::new(scheduler::TryTillSuccess::new()),
            template: tera,
            pool
        }
    }
}

fn get_server_port() -> u16 {
    env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080)
}

fn main() {
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

    let state = AppState::new(config, pool);
    let state = Arc::new(state);
    let _state = state.clone();


    /*let _server = Arbiter::start(move |ctx: &mut Context<_>| {

        weather::WeatherInquirer::new(_state)
    });

*/
    let addr = HttpServer::new(move || {
        App::with_state(state.clone())
            .middleware(middleware::Logger::default())
            .middleware(IdentityService::new(
                CookieIdentityPolicy::new(&[0; 32])
                    .name("auth-example")
                    .secure(false),
            ))
            .handler("/api/static", fs::StaticFiles::new("static/").unwrap())
            .resource("/login", |r| r.f(api::login))
            .resource("/logout", |r| r.f(api::logout))
            .resource("/", |r| r.f(api::index))
            .resource("/users", |r| r.f(api::users))
            .resource("/list", |r| r.method(http::Method::GET).with(api::list))
            .resource("/api/send_message/", |r| r.f(api::send_message))
            .resource("/api/acc_data/", |r| r.f(api::acc_data))
            .resource("/api/viber/webhook/", |r| {
                r.f(api::viber_webhook)
            })
    })
        .bind(format!("0.0.0.0:{}", get_server_port()))
        .unwrap()
        .workers(1)
        .shutdown_timeout(1)
        .start();

    let _ = sys.run();
}
