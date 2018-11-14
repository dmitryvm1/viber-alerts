use actix::Message;

#[derive(Message)]
pub struct TomorrowForecast {
    pub user_id: String
}