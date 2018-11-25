#[derive(Serialize, Deserialize, Debug)]
pub struct Bpi {
    pub USD: Usd,
    pub GBP: Usd,
    pub EUR: Usd,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BTCPrice {
    pub time: Time,
    pub disclaimer: String,
    pub chart_name: String,
    pub bpi: Bpi,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Time {
    pub updated: String,
    pub updated_iso: String,
    pub updateduk: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Usd {
    pub code: String,
    pub symbol: String,
    pub rate: String,
    pub description: String,
    pub rate_float: f64,
}
