use actix_web;
use futures::Future;
use actix_web::Error;
use actix_web::client::{ClientResponse, SendRequestError};
use std::borrow::Cow;

use viber::messages;

pub fn get_account_data(auth: &String) -> impl Future<Item=ClientResponse, Error=SendRequestError> {
    actix_web::client::get("https://chatapi.viber.com/pa/get_account_info")
        .header("X-Viber-Auth-Token", auth.clone())
        .finish()
        .unwrap()
        .send()
}

pub fn send_video_message(url: &str, size: usize, receiver: &str, auth: &String) -> impl Future<Item=ClientResponse, Error=SendRequestError> {
    let video_message = messages::VideoMessage {
        _type: Cow::from("video"),
        min_api_version: 1,
        receiver: Cow::from(receiver),
        media: Cow::from(url),
        sender: messages::Sender {
            avatar: Cow::from(""),
            name: Cow::from("Bot")
        },
        duration: 0,
        thumbnail: Cow::from(""),
        size: size,
        tracking_data: Cow::from("")
    };

    actix_web::client::post("https://chatapi.viber.com/pa/send_message")
        .header("X-Viber-Auth-Token", auth.clone())
        .json(video_message)
        .unwrap()
        .send()
}

pub fn send_file_message(url: &str, file_name: &str, size: usize, receiver: &str, auth: &String) -> impl Future<Item=ClientResponse, Error=SendRequestError> {
    let video_message = messages::FileMessage {
        _type: Cow::from("video"),
        min_api_version: 1,
        receiver: Cow::from(receiver),
        media: Cow::from(url),
        sender: messages::Sender {
            avatar: Cow::from(""),
            name: Cow::from("Bot")
        },
        file_name: Cow::from(file_name),
        size: size,
        tracking_data: Cow::from("")
    };

    actix_web::client::post("https://chatapi.viber.com/pa/send_message")
        .header("X-Viber-Auth-Token", auth.clone())
        .json(video_message)
        .unwrap()
        .send()
}

pub fn send_text_message(text: &str, receiver: &str, auth: &String) -> impl Future<Item=ClientResponse, Error=SendRequestError> {
    let text_message = messages::TextMessage {
        _type: Cow::from("text"),
        min_api_version: 1,
        receiver: Cow::from(receiver),
        text: Cow::from(text),
        sender: messages::Sender {
            avatar: Cow::from(""),
            name: Cow::from("Bot")
        },
        tracking_data: Cow::from("")
    };

    actix_web::client::post("https://chatapi.viber.com/pa/send_message")
        .header("X-Viber-Auth-Token", auth.clone())
        .json(text_message)
        .unwrap()
        .send()
}