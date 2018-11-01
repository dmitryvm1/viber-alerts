#![allow(unused_variables)]
#![feature(try_trait)]
#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
extern crate actix_web;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate json;
extern crate victoria_dom;
extern crate openssl;
extern crate futures;
extern crate actix;
extern crate env_logger;
extern crate dirs;
extern crate forecast;
extern crate reqwest;
extern crate chrono;

use chrono::prelude::*;
use forecast::*;
use std::sync::Arc;
use std::sync::Mutex;
use serde_json::Value;
use actix_web::http::{header, Method, StatusCode};
use actix_web::middleware::session::{self, RequestSession};
use actix_web::{
    error, fs, http, middleware, App, AsyncResponder, Error, HttpMessage, HttpRequest, Responder, HttpResponse, Path,
    Result,
};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::{env, io};
use futures::{Future, Stream};
use actix::{AsyncContext, Arbiter, Actor, Context, Running};
use actix_web::server::HttpServer;

static APP_NAME: &str = "viber_alerts";

pub mod viber;
pub mod config;

static LATITUDE: f64 = 50.4501;
static LONGITUDE: f64 = 30.5234;


struct Viber {
    api_key: String,
    admin_id: String
}

impl Viber {
    pub fn new(api_key: String, admin_id: String) -> Viber {
        println!("viber admin id: {}", &admin_id);
        Viber {
            api_key,
            admin_id
        }
    }

    pub fn send_text_to_admin(&self, text: &str) -> std::result::Result<(), actix_web::client::SendRequestError> {
        viber::raw::send_text_message(text, self.admin_id.as_str(), &self.api_key)
            .and_then(|response| {
                let body = response.body().poll().unwrap();
  //              println!("message  sent {:?}", body);
                Ok(())
            }).wait()
    }
}
#[derive(Debug, Serialize, Deserialize)]
struct MyObj {
    name: String,
    number: i32,
}

type AppStateType = Arc<AppState>;

fn index(req: &HttpRequest<AppStateType>) -> Box<Future<Item=HttpResponse, Error=Error>> {
    req.json()
        .from_err()  // convert all errors into `Error`
        .and_then(|val: MyObj| {
            println!("model: {:?}", val);
            Ok(HttpResponse::Ok().json(val))  // <- send response
        })
        .responder()
}

fn viber_webhook(req: &HttpRequest<AppStateType>) -> Box<Future<Item=HttpResponse, Error=Error>> {
    //  println!("Viber webhook called");
    req.payload()
        .concat2()
        .from_err()
        .and_then(|body| {
            println!("{}", std::str::from_utf8(&body).unwrap());
            Ok(HttpResponse::Ok()
                .content_type("text/plain")
                .body(""))
        }).responder()
}

fn send_message(req: &HttpRequest<AppStateType>) -> Box<Future<Item=HttpResponse, Error=Error>> {
    let state = req.state();
    let config = &state.config;
    let viber_api_key = &config.viber_api_key;
    let key = &viber_api_key.as_ref();
    viber::raw::send_text_message("Hi", config.admin_id.clone().unwrap().as_str(), key.unwrap())
        .from_err()
        .and_then(|response| {
            response.body().poll().unwrap();
            Ok(HttpResponse::Ok()
                .content_type("text/plain")
                .body("sent"))
        }).responder()
}

fn send_file_message(req: &HttpRequest<AppStateType>) -> Box<Future<Item=HttpResponse, Error=Error>> {
    let state = req.state();
    let config: &config::Config = &state.config;
    viber::raw::send_file_message(format!("{}css/styles.css", config.domain_root_url.as_ref().unwrap().as_str()).as_str(),
                                  "styles.css", 3506, config.admin_id.as_ref().unwrap().as_str() ,
                                  config.viber_api_key.as_ref().unwrap())
        .from_err()
        .and_then(|response| {
            response.body().poll().unwrap();
            Ok(HttpResponse::Ok()
                .content_type("text/plain")
                .body("sent"))
        }).responder()
}

fn acc_data(req: &HttpRequest<AppStateType>) -> Box<Future<Item=HttpResponse, Error=Error>> {
    let state = req.state();
    let config: &config::Config = &state.config;
    viber::raw::get_account_data(config.viber_api_key.as_ref().unwrap() )
        .from_err()
        .and_then(|response| {
            response.body()
                .from_err()
                .and_then(|data| {
                    let contents = String::from_utf8(data.to_vec()).unwrap();
                    Ok(HttpResponse::Ok()
                        .content_type("text/plain")
                        .body(contents))
                })
        }).responder()
}

struct WeatherInquirer {
    app_state: AppStateType
}

impl WeatherInquirer {
    fn new(app_state: AppStateType) -> WeatherInquirer {
        WeatherInquirer {
            app_state
        }
    }
}

impl WeatherInquirer {
    fn print_forecast(&self, api_response: &ApiResponse) -> Result<(), std::option::NoneError> {
        let currently = api_response.currently.as_ref()?;
        println!("Temperature now: {:?}", currently.apparent_temperature?);
        let daily = api_response.daily.as_ref()?;
        let day = daily.data.first()?;
        let dt = Utc.timestamp(day.time as i64, 0);
        let dt = dt.with_timezone(&FixedOffset::east(2*3600));
        println!("Date: {}", dt.to_rfc2822());
        println!("Temperature tomorrow: {:?}", day.temperature_low?);
        self.app_state.viber.send_text_to_admin(format!("Temperature tomorrow: {:?}", day.temperature_low?).as_str()).expect("Failed to send viber message.");
        Ok(())
    }
}

impl Actor for WeatherInquirer {
    type Context  = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(std::time::Duration::new(8, 0), |_t: &mut WeatherInquirer, _ctx: &mut Context<Self>| {
            let config = &_t.app_state.config;
            let api_key = config.dark_sky_api_key.clone();
            let reqwest_client = reqwest::Client::new();
            let api_client = forecast::ApiClient::new(&reqwest_client);
            let mut blocks = vec![ExcludeBlock::Alerts];

            let forecast_request = ForecastRequestBuilder::new(api_key.as_ref().unwrap().as_str(), LATITUDE, LONGITUDE)
                .exclude_block(ExcludeBlock::Hourly)
                .exclude_blocks(&mut blocks)
                .extend(ExtendBy::Hourly)
                .lang(Lang::Ukranian)
                .units(Units::UK)
                .build();
            let forecast_response = api_client.get_forecast(forecast_request).unwrap();
            let api_response: ApiResponse = serde_json::from_reader(forecast_response).unwrap();
            _t.print_forecast(&api_response);
        });
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        Running::Stop
    }
}

struct AppState {
    pub config: config::Config,
    pub viber: Viber
}

impl AppState {
    pub fn new(config: config::Config) -> AppState {
        let viber_api_key = config.viber_api_key.clone();
        let admin_id = config.admin_id.clone();
        AppState {
            config: config,
            viber: Viber::new(viber_api_key.unwrap(), admin_id.unwrap())
        }
    }
}

fn main() {
    env::set_var("RUST_LOG", "actix_web=debug");
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let sys = actix::System::new(APP_NAME);

    let mut privkey_path = config::Config::get_config_dir(APP_NAME);
    let mut fullchain_path = privkey_path.clone();
    privkey_path.push("privkey.pem");
    fullchain_path.push("fullchain.pem");

    // load ssl keys
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file(privkey_path.to_str().unwrap(), SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file(fullchain_path.to_str().unwrap()).unwrap();

    let _server = Arbiter::start(move|_| {
        let state = AppState::new(config::Config::read(APP_NAME));
        let state = Arc::new(state);
        let _state = state.clone();
        let addr = HttpServer::new(
            move|| {
                App::with_state(state.clone())
                    .middleware(middleware::Logger::default())
                    .resource("/api/", |r| r.f(index))
                    .resource("/api/send_message/", |r| r.f(send_message))
                    .resource("/api/send_file_message/", |r| r.f(send_file_message))
                    .resource("/api/acc_data/", |r| r.f(acc_data))
                    .resource("/api/viber/webhook", |r| r.method(http::Method::POST).f(viber_webhook))
            })
            .bind("127.0.0.1:8080")
            .unwrap().workers(2)
            .shutdown_timeout(1)
            .start();
        WeatherInquirer::new(_state)
    });

    let _ = sys.run();
}
