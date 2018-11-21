use actix_web::HttpMessage;
use futures::Future;
// use std::io::Read;

pub mod messages;
pub mod raw;

pub struct Viber {
    pub api_key: String,
    pub admin_id: String,
}

#[derive(Debug, Fail)]
#[fail(display = "Viber API error: {}", msg)]
struct ViberApiError {
    msg: String,
}

impl Viber {
    pub fn new(api_key: String, admin_id: String) -> Viber {
        Viber {
            api_key,
            admin_id
        }
    }

    pub fn update_subscribers(&self, out: &mut Vec<messages::Member>) -> std::result::Result<(), failure::Error> {
        raw::get_account_data(&self.api_key)
            .from_err()
            .and_then(|response| {
                response.body().from_err().and_then(|data| {
                    let account_info: messages::AccountInfo = serde_json::from_slice(&data)?;
                    out.clear();
                    for member in account_info.members {
                        out.push(member);
                    }
                    Ok(())
                })
            }).wait()
    }

    pub fn send_text_to(
        &self,
        text: &str,
        to: &str,
        kb: Option<messages::Keyboard>,
    ) -> std::result::Result<(), failure::Error> {
        raw::send_text_message(text, to, &self.api_key, kb)
            .from_err()
            .and_then(|response| {
                if !response.status().is_success() {
                    debug!("Failed to send message");
                }
                let body = response.body().poll()?;
                debug!("Send text to: {:?}", body);
                Ok(())
            })
            .wait()
    }

    pub fn send_file_message_to(
        &self,
        url: &str,
        name: &str,
        to: &str,
    ) -> std::result::Result<(), failure::Error> {
        raw::send_file_message(url, name, 0, to, &self.api_key)
            .from_err()
            .and_then(|response| {
                if response.status().is_success() {
                    response.body().poll()?;
                    Ok(())
                } else {
                    Err((ViberApiError {
                        msg: "error sending file msg".to_owned(),
                    })
                    .into())
                }
            })
            .wait()
    }

    pub fn send_file_message_to_admin(
        &self,
        url: &str,
        name: &str,
    ) -> std::result::Result<(), failure::Error> {
        self.send_file_message_to(url, name, self.admin_id.as_str())
    }

    pub fn send_picture_message_to(
        &self,
        url: &str,
        thumb: &str,
        text: &str,
        to: &str,
    ) -> std::result::Result<(), failure::Error> {
        raw::send_picture_message(url, text, thumb, to, &self.api_key)
            .from_err()
            .and_then(|response| {
                if response.status().is_success() {
                    response.body().poll()?;
                    Ok(())
                } else {
                    Err((ViberApiError {
                        msg: "error sending file msg".to_owned(),
                    })
                    .into())
                }
            })
            .wait()
    }

    pub fn send_picture_message_to_admin(
        &self,
        url: &str,
        thumb: &str,
        text: &str,
    ) -> std::result::Result<(), failure::Error> {
        self.send_picture_message_to(url, text, thumb, self.admin_id.as_str())
    }

    pub fn send_text_to_admin<'s>(
        &self,
        text: &str,
        kb: Option<messages::Keyboard<'s>>,
    ) -> std::result::Result<(), failure::Error> {
        self.send_text_to(text, self.admin_id.as_str(), kb)
    }
}
