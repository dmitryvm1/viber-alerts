use actix::Message;

#[derive(Message)]
pub enum WorkerUnit {
    TomorrowForecast{ user_id: String },
    ImmediateTomorrowForecast{ user_id: String, lat: f64, lon: f64 },
    BTCPrice{ user_id: String },
    UnknownCommand{ user_id: String },
}
