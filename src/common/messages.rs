use actix::Message;

#[derive(Message)]
pub enum WorkerUnit {
    TomorrowForecast{ user_id: String },
    BTCPrice{ user_id: String }
}
