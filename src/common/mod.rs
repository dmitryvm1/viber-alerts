use std::borrow::Cow;
use viber::messages::{Button, Keyboard};

pub mod messages;

pub fn get_default_keyboard<'a>() -> Keyboard<'a> {
    Keyboard {
        default_height: true,
        _type: Cow::from("keyboard"),
        buttons: vec![
            Button {
                action_body: Cow::from("bitcoin"),
                action_type: Cow::from("reply"),
                text: Cow::from("Bitcoin Price"),
                text_size: Cow::from("regular"),
            },
            Button {
                action_body: Cow::from("forecast_kiev_tomorrow"),
                action_type: Cow::from("reply"),
                text: Cow::from("Weather For Tomorrow"),
                text_size: Cow::from("regular"),
            },
        ],
    }
}


