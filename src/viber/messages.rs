use std::borrow::Cow;

#[derive(Serialize, Deserialize, Debug)]
pub struct Location {
    lat: f64,
    lon: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Member {
    pub id: String,
    pub name: String,
    pub avatar: Option<String>,
    pub role: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum EventTypes<'a> {
    Subscribed,
    Unsubscribed,
    ConversationStarted,
    Delivered,
    Failed,
    Message,
    Seen,
    #[doc(hidden)]
    Unknown(&'a str),
}


impl<'a> EventTypes<'a> {
    pub fn value(&self) -> &'a str {
        match self {
            EventTypes::Subscribed => "subscribed",
            EventTypes::Unsubscribed => "unsubscribed",
            EventTypes::ConversationStarted => "conversation_started",
            EventTypes::Delivered => "delivered",
            EventTypes::Failed => "failed",
            EventTypes::Message => "message",
            EventTypes::Seen => "seen",
            EventTypes::Unknown(s) => s,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Button<'s> {
    ActionType: Cow<'s, str>,
    ActionBody: Cow<'s, str>,
    Text: Cow<'s, str>,
    TextSize: Cow<'s, str>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Keyboard<'s> {
    Type: String,
    DefaultHeight: bool,
    Buttons: Vec<Button<'s>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountInfo {
    pub status: i64,
    pub status_message: String,
    pub id: String,
    pub name: String,
    pub uri: String,
    pub icon: String,
    pub background: String,
    pub category: String,
    pub subcategory: String,
    pub location: Location,
    pub country: String,
    pub webhook: String,
    pub event_types: Vec<String>,
    pub members: Vec<Member>,
    pub subscribers_count: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TextMessage<'s> {
    pub receiver: Cow<'s, str>,
    pub min_api_version: i64,
    pub sender: Sender<'s>,
    pub tracking_data: Cow<'s, str>,
    #[serde(rename = "type")]
    pub _type: Cow<'s, str>,
    pub keyboard: Option<Keyboard<'s>>,
    pub text: Cow<'s, str>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileMessage<'s> {
    pub receiver: Cow<'s, str>,
    pub min_api_version: i64,
    pub sender: Sender<'s>,
    pub tracking_data: Cow<'s, str>,
    #[serde(rename = "type")]
    pub _type: Cow<'s, str>,
    pub media: Cow<'s, str>,
    pub keyboard: Option<Keyboard<'s>>,
    pub size: usize,
    pub file_name: Cow<'s, str>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PictureMessage<'s> {
    pub receiver: Cow<'s, str>,
    pub min_api_version: i64,
    pub sender: Sender<'s>,
    pub tracking_data: Cow<'s, str>,
    #[serde(rename = "type")]
    pub _type: Cow<'s, str>,
    pub keyboard: Option<Keyboard<'s>>,
    pub media: Cow<'s, str>,
    pub text: Cow<'s, str>,
    pub thumbnail: Cow<'s, str>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VideoMessage<'s> {
    pub receiver: Cow<'s, str>,
    pub min_api_version: i64,
    pub sender: Sender<'s>,
    pub tracking_data: Cow<'s, str>,
    #[serde(rename = "type")]
    pub _type: Cow<'s, str>,
    pub keyboard: Option<Keyboard<'s>>,
    pub media: Cow<'s, str>,
    pub size: usize,
    pub duration: u16,
    pub thumbnail: Cow<'s, str>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Sender<'s> {
    pub name: Cow<'s, str>,
    pub avatar: Cow<'s, str>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User<'s> {
    pub id: Cow<'s, str>,
    pub name: Cow<'s, str>,
    pub avatar: Cow<'s, str>,
    pub country: Cow<'s, str>,
    pub language: Cow<'s, str>,
    pub api_version: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CallbackMessage<'s> {
    pub event: Cow<'s, str>,
    pub timestamp: u64,
    pub message_token: u64,
    pub user_id: Cow<'s, str>,
    #[serde(rename = "type")]
    pub _type: Option<Cow<'s, str>>,
    pub context: Option<Cow<'s, str>>,
    pub user: Option<User<'s>>,
    pub subscribed: Option<bool>
}
