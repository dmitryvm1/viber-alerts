#[derive(Serialize, Deserialize, Debug)]
pub struct Bpi {
    #[serde(rename="USD")]
    pub usd: Currency,
    #[serde(rename="GBP")]
    pub gbp: Currency,
    #[serde(rename="EUR")]
    pub eur: Currency,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BTCPrice {
    pub time: Time,
    pub disclaimer: String,
    #[serde(rename="chartName")]
    pub chart_name: String,
    pub bpi: Bpi,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Time {
    pub updated: String,
    #[serde(rename="updateISO")]
    pub updated_iso: String,
    pub updateduk: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Currency {
    pub code: String,
    pub symbol: String,
    pub rate: String,
    pub description: String,
    pub rate_float: f64,
}
