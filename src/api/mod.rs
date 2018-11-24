use actix_web::middleware::identity::RequestIdentity;
use actix_web::*;
use bitcoin;
use chrono::TimeZone;
use common::*;
use futures::prelude::*;
use models::NewUser;
use models::User;
use std::borrow::Borrow;
use {
    openssl::ssl::{Error as SslError, SslConnector, SslMethod},
    tokio_openssl::SslConnectorExt,
};
use std::collections::HashMap;
use std::ops::Deref;
use viber::messages::CallbackMessage;
use viber::raw;
use std::borrow::BorrowMut;
use common::messages::{ WorkerUnit };
use workers::WebWorker;
use actix::Recipient;
use actix_web::http::StatusCode;
use failure::Fail;
use futures::future::{ok as fut_ok};
use super::*;

use oauth2::basic::BasicClient;
use oauth2::prelude::*;
use oauth2::{AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
             TokenUrl};
use api::auth::GoogleProfile;
use std::time::Duration;
use workers::db::UserByEmail;
use workers::db::RegisterUser;

pub mod auth;

pub fn list(
    (state, query): (State<AppStateType>, Query<HashMap<String, String>>),
) -> Result<HttpResponse, Error> {
    let mut ctx = tera::Context::new();
    ctx.insert("text", &"Welcome!".to_owned());
    let ts = state.last_text_broadcast.read().unwrap().last_success;
    ctx.insert("last_broadcast", &chrono::Utc.timestamp(ts, 0).to_rfc2822());
    ctx.insert("members", &state.subscribers);
    let html = state.template.render("index.html", &ctx).map_err(|e| {
        error!("Template error! {:?}", e);
        error::ErrorInternalServerError("Template error")
    })?;
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

pub fn verify(req: &HttpRequest<AppStateType>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("text/html").body(""))
}

pub fn viber_webhook(
    req: &HttpRequest<AppStateType>,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    use std::borrow::Cow;

    let state = req.state();
    let addr:Addr<WebWorker> = {
        let mut temp = state.addr.lock().unwrap();
        temp.get_mut().as_ref().unwrap().clone()
    };
    let key = req.state().config.viber_api_key.clone().unwrap();
    let kb = Some(get_default_keyboard());

    req.payload()
        .concat2()
        .from_err()
        .and_then(move |body| {
            let cb_msg: Result<CallbackMessage, serde_json::Error> =
                serde_json::from_slice::<CallbackMessage>(&body);
            info!("viber hook {:?}", cb_msg);
            match cb_msg {
                Ok(ref msg) => {
                    match msg.event.as_ref() {
                        "conversation_started" => {
                            info!("message parsed {:?}", msg);
                            let user = msg.user.as_ref().unwrap();
                            raw::send_text_message(
                                "Welcome to Kiev Alerts",
                                &user.id.to_string(),
                                &key,
                                kb,
                            )
                            .wait();
                        },

                        "message" => {
                            info!("message parsed {:?}", msg);
                            let cmd = handle_user_message(&msg);
                            addr.do_send(cmd);
                        },
                        _ => {}
                    }
                    info!("sending ok response");
                    Ok(HttpResponse::Ok().content_type("text/plain").body(""))
                }
                Err(e) => {
                    debug!("Error parsing json, {:?}", e);
                    Ok(HttpResponse::Ok().content_type("text/plain").body(""))
                }
            }
        })
        .responder()
}

fn handle_user_message(msg: &CallbackMessage) -> WorkerUnit {
    let user = msg.sender.as_ref().unwrap().id.as_ref().unwrap();
    let message = msg.message.as_ref().unwrap();
    let actor_message = match msg.message.as_ref().unwrap()._type.as_ref() {
        "location" => {
            let location = msg.message.as_ref().unwrap().location.as_ref().unwrap();
            WorkerUnit::ImmediateTomorrowForecast { user_id: user.to_string(), lat: location.lat, lon: location.lon }
        },
        "text" => {
            match message.text.as_ref().unwrap().as_ref() {
                "bitcoin" => {
                    WorkerUnit::BTCPrice { user_id: user.to_string() }
                },
                "forecast_kiev_tomorrow" => {
                    info!("message parsed {:?}", msg);
                    WorkerUnit::TomorrowForecast { user_id: user.to_string() }
                }
                _ => WorkerUnit::UnknownCommand { user_id: user.to_string() }
            }
        },
        _ => WorkerUnit::UnknownCommand { user_id: user.to_string() }
    };
    actor_message
}

pub fn send_message(
    req: &HttpRequest<AppStateType>,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let state = req.state();
    let config = &state.config;
    let viber_api_key = &config.viber_api_key;
    let key = &viber_api_key.as_ref();
    super::viber::raw::send_text_message(
        "Hi",
        config.admin_id.as_ref().unwrap().as_str(),
        key.unwrap(),
        None,
    )
    .from_err()
    .and_then(|response| {
        response.body().poll()?;
        Ok(HttpResponse::Ok().content_type("text/plain").body("sent"))
    })
    .responder()
}

pub fn acc_data(
    req: &HttpRequest<AppStateType>,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let state = req.state();
    let config: &super::config::Config = &state.config;
    super::viber::raw::get_account_data(config.viber_api_key.as_ref().unwrap())
        .from_err()
        .and_then(|response| {
            response.body().from_err().and_then(|data| {
                let contents = String::from_utf8(data.to_vec()).unwrap_or("".to_owned());
                Ok(HttpResponse::Ok().content_type("text/plain").body(contents))
            })
        })
        .responder()
}

pub fn google_oauth(req: &HttpRequest<AppStateType>) -> Box<Future<Item=HttpResponse, Error=Error>>  {
    use futures::{
        future::{
            ok as fut_ok,
            err as fut_err
        }
    };
    let code = AuthorizationCode::new(req.query().get("code").unwrap().to_string());
    let token = {
        let state: &AppStateType = req.state();
        let st = state;
        let mut client = st.auth_client.lock().unwrap();
        client.get_mut().as_ref().unwrap().exchange_code(code).map_err(|e|{
            actix_web::error::ErrorInternalServerError(e)
        })
    };
    if token.is_err() {
        return fut_err(actix_web::error::ErrorUnauthorized("could not exchange token")).responder();
    }
    let resp = reqwest::get(&format!("https://www.googleapis.com/userinfo/v2/me?access_token={}", token.unwrap().access_token().secret()));
    if !resp.is_ok() {
        return fut_err(actix_web::error::ErrorUnauthorized("Could not get user info.")).responder();
    }
    let json: GoogleProfile = serde_json::from_reader(resp.unwrap()).expect("bad gauth response");
    req.remember(json.email.expect("no email"));
    fut_ok(HttpResponse::Found().header("location", "/api/").finish()).responder()
}

pub fn index(req: &HttpRequest<AppStateType>) -> Result<HttpResponse, Error> {
    let state: &AppStateType = req.state();
    let mut addr = state.addr.lock().unwrap();
    let addr = addr.get_mut().as_ref().unwrap().clone();
    if req.identity().is_none() {
        let mut ctx = tera::Context::new();
        /*ctx.insert("app_name", "Viber Alerts");
        let html = state.read().unwrap().template.render("oauth_login.html", &ctx).map_err(|e| {
            error!("Template error! {:?}", e);
            error::ErrorInternalServerError("Template error")
        })?;*/
        let st = state;
        let mut client = st.auth_client.lock().unwrap();
        let (authorize_url, csrf_state) = client.get_mut().as_ref().unwrap().authorize_url(CsrfToken::new_random);
        debug!("{:?}", authorize_url);
        ctx.insert("app_name", &"Viber Alerts!".to_owned());
        ctx.insert("auth_url", &authorize_url.to_string());
        let html = state.template.render("login.html", &ctx).map_err(|e| {
            error!("Template error! {:?}", e);
            error::ErrorInternalServerError("Template error")
        })?;
        Ok(HttpResponse::Ok().content_type("text/html").body(html))
    } else {
        let mut ctx = tera::Context::new();

        let ts = state.last_text_broadcast.read().unwrap().last_success;
        ctx.insert("last_broadcast", &chrono::Utc.timestamp(ts, 0).to_rfc2822());
        ctx.insert("members", &state.subscribers);
        let user_email = req.identity().unwrap();

        let result = addr.send(UserByEmail(user_email.clone())).wait();
        let user = {
            if result.is_err() {
                addr.send(RegisterUser(user_email)).wait().unwrap()
            } else {
                result.unwrap()
            }
        }.unwrap();


        ctx.insert("email", user.email.as_ref().unwrap());
        ctx.insert("verified", &user.viber_id.is_some());
        let html = state.template.render("index.html", &ctx).map_err(|e| {
            error!("Template error! {:?}", e);
            error::ErrorInternalServerError("Template error")
        })?;
        Ok(HttpResponse::Ok().content_type("text/html").body(html))
    }
}

#[derive(Deserialize)]
pub struct LoginParams {
    email_or_name: String,
    password: String,
}

pub fn users(req: &HttpRequest<AppStateType>) -> HttpResponse {
    let pool = &req.state().pool;
    let users = User::all(pool.get().unwrap().deref()).unwrap();
    HttpResponse::Ok().body(format!("{:?}", users))
}

pub fn logout(req: &HttpRequest<AppStateType>) -> HttpResponse {
    req.forget();
    HttpResponse::Found().header("location", "/api/").finish()
}
