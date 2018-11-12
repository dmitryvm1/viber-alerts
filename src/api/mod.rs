use super::AppStateType;
use actix_web::*;
use std::collections::HashMap;
use models::Post;
use models::NewPost;
use futures::prelude::*;
use chrono::TimeZone;
use std::ops::Deref;
use viber::*;
use std::borrow::Borrow;
use actix_web::middleware::identity::RequestIdentity;



pub fn list(
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

pub fn viber_webhook(
    req: &HttpRequest<AppStateType>,
) ->  Box<Future<Item = HttpResponse, Error = Error>> {
    let key = req.state().viber.lock().unwrap().api_key.clone();

    req.payload()
        .concat2()
        .from_err()
        .and_then(move|body| {

            let cb_msg: Result<messages::CallbackMessage, serde_json::Error>  = serde_json::from_slice(&body);
            match cb_msg
            {
                Ok(ref msg) => {
                    info!("message parsed {:?}", msg);
                    if msg.event.eq(&std::borrow::Cow::from("conversation_started")) {
                        let user = msg.user.as_ref().unwrap();
                        raw::send_text_message("Hi", &user.id.to_string(), &key , None).wait();

                    }
                    Ok(HttpResponse::Ok().content_type("text/plain").body(""))
                },
                Err(e) => {
                    debug!("Error parsing json, {:?}", e);
                    Ok(HttpResponse::Ok().content_type("text/plain").body(""))
                }
            }
        }).responder()

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
        key.unwrap(), None
    )
        .from_err()
        .and_then(|response| {
            response.body().poll()?;
            Ok(HttpResponse::Ok().content_type("text/plain").body("sent"))
        })
        .responder()
}

pub fn acc_data(req: &HttpRequest<AppStateType>) -> Box<Future<Item = HttpResponse, Error = Error>> {
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

pub fn index(req: &HttpRequest<AppStateType>) -> String {
    format!("Hello {}", req.identity().unwrap_or("Anonymous".to_owned()))
}

pub fn login(req: &HttpRequest<AppStateType>) -> HttpResponse {
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

pub fn users(req: &HttpRequest<AppStateType>) -> HttpResponse {
    let pool = &req.state().pool;
    let users = Post::all(pool.get().unwrap().deref()).unwrap();
    HttpResponse::Ok().body(format!("{:?}", users))
}

pub fn logout(req: &HttpRequest<AppStateType>) -> HttpResponse {
    req.forget();
    HttpResponse::Found().header("location", "/").finish()
}