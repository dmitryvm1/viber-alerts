use chrono::*;

pub struct TryTillSuccess {
    pub last_success: i64,
    from_hour: u16,
    to_hour: u16,
}

impl TryTillSuccess {
    pub fn new() -> TryTillSuccess {
        TryTillSuccess {
            last_success: 0,
            from_hour: 0,
            to_hour: 24,
        }
    }

    pub fn daily(&mut self, from: u8, to: u8, f: &mut FnMut() -> bool) -> &Self {
        let now = Utc::now();
        let past_event = Utc.timestamp(self.last_success, 0);
        let lower_edge = Utc::now().with_hour(from as u32).unwrap();
        if past_event.timestamp() < lower_edge.timestamp() && now.hour() < to as u32 {
            debug!("Executing at hour: {}", now.hour());
            if f() {
                self.last_success = now.timestamp();
            }
            // check if success -> change past event
        }
        self
    }
}
