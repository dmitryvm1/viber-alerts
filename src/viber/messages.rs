use std::borrow::Cow;

#[derive(Serialize, Deserialize)]
pub struct Location {
    lat: f64,
    lon: f64,
}

#[derive(Serialize, Deserialize)]
pub struct Members {
    id: String,
    name: String,
    avatar: String,
    role: String,
}

pub enum EventTypes<'a>  {
  Subscribed,
  Unsubscribed,
  ConversationStarted,
  Delivered,
  Failed,
  Message,
  Seen,
  #[doc(hidden)]
  Unknown(&'a str)
}

impl<'a> EventTypes<'a>{
    pub fn value(&self) -> &'a str {
        match self {
            EventTypes::Subscribed => "subscribed",
            EventTypes::Unsubscribed => "unsubscribed",
            EventTypes::ConversationStarted => "conversation_started",
            EventTypes::Delivered => "delivered",
            EventTypes::Failed => "failed",
            EventTypes::Message => "message",
            EventTypes::Seen => "seen",
            EventTypes::Unknown(s) => s
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AccountInfo {
    status: i64,
    status_message: String,
    id: String,
    name: String,
    uri: String,
    icon: String,
    background: String,
    category: String,
    subcategory: String,
    location: Location,
    country: String,
    webhook: String,
    event_types: Vec<String>,
    members: Vec<Members>,
    subscribers_count: i64,
}

#[derive(Serialize, Deserialize)]
pub struct TextMessage<'s> {
    pub receiver: Cow<'s, str>,
    pub min_api_version: i64,
    pub sender: Sender<'s>,
    pub tracking_data: Cow<'s, str>,
    #[serde(rename = "type")]
    pub _type: Cow<'s, str>,
    pub text: Cow<'s, str>,
}

#[derive(Serialize, Deserialize)]
pub struct FileMessage<'s> {
    pub receiver: Cow<'s, str>,
    pub min_api_version: i64,
    pub sender: Sender<'s>,
    pub tracking_data: Cow<'s, str>,
    #[serde(rename = "type")]
    pub _type: Cow<'s, str>,
    pub media: Cow<'s, str>,
    pub size: usize,
    pub file_name: Cow<'s, str>
}

#[derive(Serialize, Deserialize)]
pub struct VideoMessage<'s> {
    pub receiver: Cow<'s, str>,
    pub min_api_version: i64,
    pub sender: Sender<'s>,
    pub tracking_data: Cow<'s, str>,
    #[serde(rename = "type")]
    pub _type: Cow<'s, str>,
    pub media: Cow<'s, str>,
    pub size: usize,
    pub duration: u16,
    pub thumbnail: Cow<'s, str>
}

#[derive(Serialize, Deserialize)]
pub struct Sender<'s> {
    pub name: Cow<'s, str>,
    pub avatar: Cow<'s, str>,
}
