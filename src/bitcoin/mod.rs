use actix_web::client;
use actix_web::client::{ClientResponse, SendRequestError};
use futures::Future;
use serde_json;
use actix_web::HttpMessage;
use self::types::*;
// use serde_json::*;

pub mod types;

pub fn get_bitcoin_price() -> Option<BTCPrice> {
    let response = client::get("https://api.coindesk.com/v1/bpi/currentprice.json")
        .finish().unwrap()
        .send().wait();
    if response.is_err() {
        error!("request failed for get_bitcoin_price");
        return None
    }
    response.unwrap().body()
        .and_then(|data| {
            let price:Option<BTCPrice> = serde_json::from_slice(&data).ok();
            Ok(price)
        }).wait().unwrap()
}