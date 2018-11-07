use forecast::ApiResponse;
use AppStateType;
use chrono::*;
use forecast::*;
use std::io::Read;
use futures::Future;
use std::fs::File;
use actix_web::HttpMessage;
use std::io::Write;
use actix_web::*;


static LATITUDE: f64 = 50.4501;
static LONGITUDE: f64 = 30.5234;

#[derive(Debug, Fail)]
enum JsonError {
    #[fail(display = "field is missing: {}", name)]
    MissingField {
        name: String,
    },
    #[fail(display = "error accessing array")]
    ArrayIndex,
}

#[derive(Debug, Fail)]
#[fail(display = "Custom error: {}", msg)]
struct CustomError {
    msg: String
}

pub struct WeatherInquirer {
    pub app_state: AppStateType,
    pub last_response: Option<ApiResponse>,
    pub last_subscriber_update: i64
}

impl WeatherInquirer {
    pub fn new(app_state: AppStateType) -> WeatherInquirer {
        WeatherInquirer {
            app_state,
            last_response: None,
            last_subscriber_update: 0
        }
    }
}

impl WeatherInquirer {
    pub fn inquire_if_needed(&mut self) -> Result<bool, failure::Error> {
        if self.last_response.is_none() {
            self.last_response = self.inquire().map_err(|e| {
                error!("Error while requesting forecast: {:?}", e.as_fail())
            }).ok();
            return Ok(true);
        } else {
            let today = Utc::now();
            // check if the second daily forecast is for today:
            let dt = {
                let lr = self.last_response.as_ref().unwrap();
                let daily = lr.daily.as_ref().ok_or(JsonError::MissingField { name: "daily".to_owned() })?;
                let first = daily.data.get(1).ok_or(JsonError::ArrayIndex)?;
                Utc.timestamp(first.time as i64, 0)
            };
            if dt.day() == today.day() {
                return Ok(false);
            } else {
                self.last_response = self.inquire().map_err(|e| {
                    error!("Error while requesting forecast: {:?}", e.as_fail())
                }).ok();
                return Ok(true);
            }
        }
    }

    pub fn download_image(&self, name: &str) -> Result<(), actix_web::error::Error> {

        client::get(format!("{}weather/kiev/{}", self.app_state.config.domain_root_url.as_ref().unwrap(), name))
            .finish().unwrap()
            .send()
            .from_err()
            .and_then(move|response|{
                response.body()
                    .from_err()
                    .and_then(move|data| {
                        if response.status().is_success() {
                            let mut file = File::create(format!("static/{}", name))?;
                            file.write_all(&data.to_vec()).map_err(|e| {
                                error::ErrorInternalServerError("Failed to write weather image.")
                            })
                        } else {
                            Ok(())
                        }
                    })
            })
            .wait()
    }

    #[allow(dead_code)]
    fn today(&self) -> Result<&DataPoint, failure::Error> {
        if let Some(ref lr) = self.last_response {
            let daily = lr.daily.as_ref().ok_or(JsonError::MissingField { name: "daily".to_owned() })?;
            let first = daily.data.get(1);
            return first.ok_or(failure::Error::from(JsonError::ArrayIndex));
        }
        Err(failure::Error::from(CustomError { msg: "Forecast data is not present.".to_owned() }))
    }

    fn tomorrow(&self) -> Result<&DataPoint, failure::Error> {
        if let Some(ref lr) = self.last_response {
            let daily = lr.daily.as_ref().ok_or(JsonError::MissingField { name: "daily".to_owned() })?;
            let second = daily.data.get(2);
            return second.ok_or(failure::Error::from(JsonError::ArrayIndex));
        }
        Err(failure::Error::from(CustomError { msg: "Forecast data is not present.".to_owned() }))
    }

    fn inquire(&self) -> Result<ApiResponse, failure::Error> {
        let config = &self.app_state.config;
        let api_key = &config.dark_sky_api_key;
        let reqwest_client = reqwest::Client::new();
        let api_client = forecast::ApiClient::new(&reqwest_client);
        let mut blocks = vec![ExcludeBlock::Alerts];

        let forecast_request = ForecastRequestBuilder::new(api_key.as_ref().unwrap().as_str(), LATITUDE, LONGITUDE)
            .exclude_block(ExcludeBlock::Hourly)
            .exclude_blocks(&mut blocks)
            .extend(ExtendBy::Hourly)
            .lang(Lang::Ukranian)
            .units(Units::UK)
            .build();
        info!("Requesting weather forecast");
        let mut forecast_response = api_client.get_forecast(forecast_request)?;
        if !forecast_response.status().is_success() {
            let mut body = String::new();
            forecast_response.read_to_string(&mut body)?;
            return Err(failure::Error::from(CustomError { msg: format!("Dark sky response failure: {}", body) }));
        }
        serde_json::from_reader(forecast_response).map_err(|e| {
            failure::Error::from(e)
        })
    }

    fn should_broadcast(&self) -> bool {
        let now = Utc::now().with_timezone(&FixedOffset::east(2*3600));
        let since_last_bc = now.timestamp() - *self.app_state.last_broadcast.read().unwrap();
        debug!("Since last broadcast: {}", since_last_bc);
        if (since_last_bc  > 60 * 60 * 24) && (now.hour() >= 14 && now.hour() <= 23) {
            return true;
        }
        debug!("Should broadcast: false. Hour: {}", now.hour());
        false
    }

    pub fn send_image(&self) -> Result<(), failure::Error> {
        use std::path;
        let date = Utc::now();
        let name = format!("{}-{}-{}.jpg", date.year(), date.month(), date.day());
        let thumb = format!("{}-{}-{}t.jpg", date.year(), date.month(), date.day());
        let file_path = format!("static/{}", &name);
        let path = path::Path::new(file_path.as_str());
        if path.exists() {
            let url = format!("{}api/static/{}", self.app_state.config.hosting_root_url.as_ref().unwrap(), &name);
            let thumb_url = format!("{}api/static/{}", self.app_state.config.hosting_root_url.as_ref().unwrap(), &thumb);
            self.app_state.viber.lock().unwrap().send_picture_message_to_admin(url.as_str(), thumb_url.as_str(), "Прогноз на 7 дней")
        } else {
            Err((CustomError { msg: "no image to send for today".to_owned()}).into())
        }
    }
    pub fn broadcast_forecast(&mut self) -> Result<(), failure::Error> {
        if !self.should_broadcast() {
            return Ok(());
        }
        if self.send_image().is_err() {
            error!("no file msg");
        }
        {
            let day = self.tomorrow()?;
            let dt = Utc.timestamp(day.time as i64, 0);
            let (precip, probability) = match day.precip_type.as_ref() {
                Some(p) => {
                    let pr = match p {
                        PrecipType::Rain => "Дождь",
                        PrecipType::Snow => "Снег",
                        PrecipType::Sleet => "Дождь со снегом"
                    };
                    (pr, day.precip_probability.unwrap())
                },
                None => ("-", 0.0)
            };
            let msg = format!("Прогноз на завтра {}.{}: \nТемпература: {:?} - {:?} \nОсадки: {:?} с вероятностью {}%", dt.day(),
                              dt.month(),
                              day.temperature_low.ok_or(
                                  JsonError::MissingField { name: "temperature_low".to_owned() }
                              )?,
                              day.temperature_high.ok_or(
                                  JsonError::MissingField { name: "temperature_high".to_owned() }
                              )?, precip, probability * 100.0);
            info!("Sending viber message");
            // self.app_state.viber.lock().unwrap().broadcast_text(msg.as_str())?;
            self.app_state.viber.lock().unwrap().send_text_to_admin(msg.as_str())?;
        }
        {
            let st = &self.app_state;
            *st.last_broadcast.write().unwrap() = Utc::now().with_timezone(&FixedOffset::east(2 * 3600)).timestamp();
        }
        Ok(())
    }
}