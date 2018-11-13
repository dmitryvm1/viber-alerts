use actix_web::client;
use futures::Future;
use serde_json;
use actix_web::HttpMessage;
use self::types::*;
use actix_web::error::{PayloadError, Error};
// use serde_json::*;
use weather::CustomError;
use failure;

pub mod types;

pub fn get_bitcoin_price() -> Result<Option<BTCPrice>, failure::Error> {
    let response = client::get("https://api.coindesk.com/v1/bpi/currentprice.json")
        .finish().unwrap()
        .send().wait();
    response?.body()
        .from_err()
        .and_then(|data| {
            Ok(serde_json::from_slice(&data).ok())
        }).wait()
}