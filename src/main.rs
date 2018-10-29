#![allow(unused_variables)]
#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
extern crate reqwest;
extern crate actix_web;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate json;
extern crate victoria_dom;
extern crate openssl;
extern crate futures;
extern crate actix;
extern crate env_logger;
extern crate dirs;

use std::io::Read;
use serde_json::Value;
use actix_web::http::{header, Method, StatusCode};
use actix_web::middleware::session::{self, RequestSession};
use actix_web::{
    error, fs, http, middleware, pred, server, App, AsyncResponder, Error, HttpMessage, HttpRequest, Responder, HttpResponse, Path,
    Result,
};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use futures::future::{result, FutureResult};
use std::{env, io};
use futures::{Future, Stream};

static LATITUDE: f32 = 50.4501;
static LONGITUDE: f32 = 30.5234;
static APP_NAME: &str = "viber_alerts";

pub mod viber;
pub mod config;

#[derive(Debug, Serialize, Deserialize)]
struct MyObj {
    name: String,
    number: i32,
}

/// This handler uses `HttpRequest::json()` for loading json object.
/// 
fn index(req: &HttpRequest<AppState>) -> Box<Future<Item=HttpResponse, Error=Error>> {
    req.json()
        .from_err()  // convert all errors into `Error`
        .and_then(|val: MyObj| {
            println!("model: {:?}", val);
            Ok(HttpResponse::Ok().json(val))  // <- send response
        })
        .responder()
}

fn viber_webhook(req: &HttpRequest<AppState>) -> Box<Future<Item=HttpResponse, Error=Error>> {
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

fn send_message(req: &HttpRequest<AppState>) -> Box<Future<Item=HttpResponse, Error=Error>> {
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

fn send_file_message(req: &HttpRequest<AppState>) -> Box<Future<Item=HttpResponse, Error=Error>> {
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

fn acc_data(req: &HttpRequest<AppState>) -> Box<Future<Item=HttpResponse, Error=Error>> {
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

struct AppState {
    pub config: config::Config
}

fn main() {
    // load ssl keys
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

    // let viber_arbiter = actix::Arbiter::new();
    let addr = server::new(
        || {
            let state = AppState {
                config: config::Config::read(APP_NAME)
            };
            App::with_state(state)
                .middleware(middleware::Logger::default())
                .resource("/api/", |r| r.f(index))
                .resource("/api/send_message/", |r| r.f(send_message))
                .resource("/api/send_file_message/", |r| r.f(send_file_message))
                .resource("/api/acc_data/", |r| r.f(acc_data))
                .resource("/api/viber/webhook", |r| r.method(http::Method::POST).f(viber_webhook))
        })
        .bind("127.0.0.1:8080")
        .unwrap()
        .shutdown_timeout(1)
        .start();
    let _ = sys.run();
}
