use actix_web;
use actix_web::client::{ClientResponse, SendRequestError};
use futures::Future;
use std::borrow::Cow;

use viber::messages;

pub fn get_account_data(
    auth: &String,
) -> impl Future<Item = ClientResponse, Error = SendRequestError> {
    actix_web::client::get("https://chatapi.viber.com/pa/get_account_info")
        .header("X-Viber-Auth-Token", auth.clone())
        .finish()
        .unwrap()
        .send()
}

pub fn send_video_message(
    url: &str,
    size: usize,
    receiver: &str,
    auth: &String,
) -> impl Future<Item = ClientResponse, Error = SendRequestError> {
    let video_message = messages::VideoMessage {
        _type: Cow::from("video"),
        min_api_version: 1,
        receiver: Cow::from(receiver),
        media: Cow::from(url),
        sender: messages::Sender {
            avatar: Cow::from(""),
            name: Cow::from("Bot"),
            id: None,
            language: None,
            country: None,
            api_version: None
        },
        keyboard: None,
        duration: 0,
        thumbnail: Cow::from(""),
        size: size,
        tracking_data: Cow::from(""),
    };

    actix_web::client::post("https://chatapi.viber.com/pa/send_message")
        .header("X-Viber-Auth-Token", auth.clone())
        .json(video_message)
        .unwrap()
        .send()
}

pub fn send_file_message(
    url: &str,
    file_name: &str,
    size: usize,
    receiver: &str,
    auth: &String,
) -> impl Future<Item = ClientResponse, Error = SendRequestError> {
    let file_message = messages::FileMessage {
        _type: Cow::from("file"),
        min_api_version: 1,
        receiver: Cow::from(receiver),
        media: Cow::from(url),
        sender: messages::Sender {
            avatar: Cow::from(""),
            name: Cow::from("Bot"),
            id: None,
            language: None,
            country: None,
            api_version: None
        },
        keyboard: None,
        file_name: Cow::from(file_name),
        size: size,
        tracking_data: Cow::from(""),
    };

    actix_web::client::post("https://chatapi.viber.com/pa/send_message")
        .header("X-Viber-Auth-Token", auth.clone())
        .json(file_message)
        .unwrap()
        .send()
}

pub fn send_picture_message(
    url: &str,
    thumb: &str,
    text: &str,
    receiver: &str,
    auth: &String,
) -> impl Future<Item = ClientResponse, Error = SendRequestError> {
    let picture_message = messages::PictureMessage {
        _type: Cow::from("picture"),
        min_api_version: 1,
        receiver: Cow::from(receiver),
        media: Cow::from(url),
        sender: messages::Sender {
            avatar: Cow::from(""),
            name: Cow::from("Bot"),
            id: None,
            language: None,
            country: None,
            api_version: None
        },
        keyboard: None,
        text: Cow::from(text),
        thumbnail: Cow::from(thumb),
        tracking_data: Cow::from(""),
    };
    debug!("{:?}", picture_message);
    actix_web::client::post("https://chatapi.viber.com/pa/send_message")
        .header("X-Viber-Auth-Token", auth.clone())
        .json(picture_message)
        .unwrap()
        .send()
}

pub fn send_text_message(
    text: &str,
    receiver: &str,
    auth: &String,
    kb: Option<messages::Keyboard>
) -> impl Future<Item = ClientResponse, Error = SendRequestError> {
    let text_message = messages::TextMessage {
        _type: Cow::from("text"),
        min_api_version: 1,
        receiver: Cow::from(receiver),
        text: Cow::from(text),
        keyboard: kb,
        sender: messages::Sender {
            avatar: Cow::from(""),
            name: Cow::from("Bot"),
            id: None,
            language: None,
            country: None,
            api_version: None
        },
        tracking_data: Cow::from(""),
    };

    actix_web::client::post("https://chatapi.viber.com/pa/send_message")
        .header("X-Viber-Auth-Token", auth.clone())
        .json(text_message)
        .unwrap()
        .send()
}
