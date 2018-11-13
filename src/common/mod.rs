use viber::messages::{Button, Keyboard};
use std::borrow::Cow;

pub fn get_default_keyboard<'a>() -> Keyboard<'a> {
    Keyboard {
        DefaultHeight: true,
        Type: Cow::from("keyboard"),
        Buttons: vec!(Button {
            ActionBody: Cow::from("bitcoin"),
            ActionType: Cow::from("reply"),
            Text: Cow::from("Bitcoin Price"),
            TextSize: Cow::from("regular")
        })
    }
}