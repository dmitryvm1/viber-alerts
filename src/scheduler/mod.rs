use chrono::*;

pub struct TryTillSuccess {
    pub last_success: i64,
}

impl TryTillSuccess {
    pub fn new() -> TryTillSuccess {
        TryTillSuccess {
            last_success: 0,
        }
    }

    pub fn daily(&mut self, from: u8, to: u8, f: &mut FnMut() -> bool) -> &Self {
        let now = Utc::now();
        let past_event = Utc.timestamp(self.last_success, 0);
        let lower_edge = Utc::now().with_hour(from as u32).unwrap();
        if lower_edge.timestamp() - past_event.timestamp()
            > ((24 - to as i64 + from as i64) * 60 * 60)
            && now.hour() < to as u32
            && now.hour() > from as u32
        {
            if f() {
                self.last_success = now.timestamp();
            }
            // check if success -> change past event
        }
        self
    }
}
