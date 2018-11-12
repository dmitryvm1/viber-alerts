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
struct Button {
    ActionType: String,
    ActionBody: String,
    Text: String,
    TextSize: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Keyboard {
    Type: String,
    DefaultHeight: bool,
    Buttons: Vec<Button>,
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
    pub keyboard: Option<Keyboard>,
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
    pub keyboard: Option<Keyboard>,
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
    pub keyboard: Option<Keyboard>,
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
    pub keyboard: Option<Keyboard>,
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
