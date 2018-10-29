

pub struct DarkSky {
    api_key: String
}

impl DarkSky {
    pub fn new(api_key: &str)-> DarkSky {
        DarkSky {
            api_key: api_key
        }
    }

    fn weather(&self) {
        let mut res = reqwest::get(&format!("https://api.darksky.net/forecast/{}/{},{}",self.api_key, LATITUDE, LONGITUDE)).unwrap();
        let mut buffer = String::new();
        let mut result = res.read_to_string(&mut buffer);
        let v: Value = serde_json::from_str(&buffer).unwrap();
        for k in v.as_object().unwrap().keys() {
            println!("{}", k);
        }
        println!("{}", v);
    }
}