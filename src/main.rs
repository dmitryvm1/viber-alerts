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

use actix_web::middleware::identity::RequestIdentity;
use actix_web::middleware::identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{
    http, middleware, App, AsyncResponder, Error, HttpMessage, HttpRequest, HttpResponse, Query,
    State,
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

static APP_NAME: &str = "viber_alerts";

pub mod config;
pub mod models;
pub mod scheduler;
pub mod schema;
pub mod viber;
pub mod weather;

#[cfg(debug_assertions)]
static QUERY_INTERVAL: u64 = 6;
#[cfg(not(debug_assertions))]
static QUERY_INTERVAL: u64 = 60;

pub type AppStateType = Arc<AppState>;
type PgPool = Pool<ConnectionManager<PgConnection>>;

fn list(
    (state, query): (State<AppStateType>, Query<HashMap<String, String>>),
) -> Result<HttpResponse, Error> {
    // let s = if let Some(name) = query.get("name") {
    // <- submitted form
    let mut ctx = tera::Context::new();
    //  ctx.add("name", &name.to_owned());
    ctx.insert("text", &"Welcome!".to_owned());
    let ts = state.last_text_broadcast.read().unwrap().last_success;

    ctx.insert("last_broadcast", &chrono::Utc.timestamp(ts, 0).to_rfc2822());
    ctx.insert("members", &state.viber.lock().unwrap().subscribers);
    let s = state.template.render("index.html", &ctx).map_err(|e| {
        error!("Template error! {:?}", e);
        error::ErrorInternalServerError("Template error")
    })?;

    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

fn viber_webhook(
    req: &HttpRequest<AppStateType>,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    req.payload()
        .concat2()
        .from_err()
        .and_then(|body| {
            info!("{}", std::str::from_utf8(&body)?);
            Ok(HttpResponse::Ok().content_type("text/plain").body(""))
        })
        .responder()
}

fn send_message(
    req: &HttpRequest<AppStateType>,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let state = req.state();
    let config = &state.config;
    let viber_api_key = &config.viber_api_key;
    let key = &viber_api_key.as_ref();
    viber::raw::send_text_message(
        "Hi",
        config.admin_id.as_ref().unwrap().as_str(),
        key.unwrap(),
    )
    .from_err()
    .and_then(|response| {
        response.body().poll()?;
        Ok(HttpResponse::Ok().content_type("text/plain").body("sent"))
    })
    .responder()
}

fn acc_data(req: &HttpRequest<AppStateType>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let state = req.state();
    let config: &config::Config = &state.config;
    viber::raw::get_account_data(config.viber_api_key.as_ref().unwrap())
        .from_err()
        .and_then(|response| {
            response.body().from_err().and_then(|data| {
                let contents = String::from_utf8(data.to_vec()).unwrap_or("".to_owned());
                Ok(HttpResponse::Ok().content_type("text/plain").body(contents))
            })
        })
        .responder()
}

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
                    Ok(q) => {
                        if q {
                            let date = chrono::Utc::now();
                            let name =
                                format!("{}-{}-{}.jpg", date.year(), date.month(), date.day());
                            _t.download_image(name.as_str()).map_err(|e| {
                                warn!("Image not downloaded. {:?}", e);
                            });
                            let name =
                                format!("{}-{}-{}t.jpg", date.year(), date.month(), date.day());
                            _t.download_image(name.as_str()).map_err(|e| {
                                warn!("Image not downloaded. {:?}", e);
                            });
                            if _t
                                .app_state
                                .viber
                                .lock()
                                .unwrap()
                                .update_subscribers()
                                .is_err()
                            {
                                warn!("Failed to read subscribers.");
                            }
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
            pool,
        }
    }
}

fn get_server_port() -> u16 {
    env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080)
}

fn index(req: &HttpRequest<AppStateType>) -> String {
    format!("Hello {}", req.identity().unwrap_or("Anonymous".to_owned()))
}

fn login(req: &HttpRequest<AppStateType>) -> HttpResponse {
    {
        let q = req.query();
        let user = q.get("user").unwrap();
        let password = q.get("password").unwrap();
        println!("u: {}, p: {}", user, password);
    }
    {
        let pool = &req.state().pool;
        let new_post = NewPost {
            body: "test",
            title: "title",
        };
        Post::insert(new_post, pool.get().unwrap().deref()).unwrap_or_else(|e| {
            error!("Failed to insert post");
            0
        });
    }

    req.remember("user1".to_owned());
    HttpResponse::Found().header("location", "/").finish()
}

fn users(req: &HttpRequest<AppStateType>) -> HttpResponse {
    let pool = &req.state().pool;
    let users = Post::all(pool.get().unwrap().deref()).unwrap();

    HttpResponse::Ok().body(format!("{:?}", users))
}

fn logout(req: &HttpRequest<AppStateType>) -> HttpResponse {
    req.forget();
    HttpResponse::Found().header("location", "/").finish()
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

    let _server = Arbiter::start(move |_| {
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

        let addr = HttpServer::new(move || {
            App::with_state(state.clone())
                .middleware(middleware::Logger::default())
                .middleware(IdentityService::new(
                    CookieIdentityPolicy::new(&[0; 32])
                        .name("auth-example")
                        .secure(false),
                ))
                .handler("/api/static", fs::StaticFiles::new("static/").unwrap())
                .resource("/login", |r| r.f(login))
                .resource("/logout", |r| r.f(logout))
                .resource("/", |r| r.f(index))
                .resource("/users", |r| r.f(users))
                .resource("/list", |r| r.method(http::Method::GET).with(list))
                .resource("/api/send_message/", |r| r.f(send_message))
                .resource("/api/acc_data/", |r| r.f(acc_data))
                .resource("/api/viber/webhook", |r| {
                    r.method(http::Method::POST).f(viber_webhook)
                })
        })
        .bind(format!("0.0.0.0:{}", get_server_port()))
        .unwrap()
        .workers(1)
        .shutdown_timeout(1)
        .start();
        weather::WeatherInquirer::new(_state)
    });

    let _ = sys.run();
}
