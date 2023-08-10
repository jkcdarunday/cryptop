extern crate serde_json;
extern crate ureq;

use std::error::Error;

pub struct CryptoPrice {
    pub name: String,
    pub symbol: String,
    pub price: f64,
    pub change: f64,
    pub market_cap: f64,
    pub volume_24h: f64,
}

pub fn get_top_cryptos() -> Result<Vec<CryptoPrice>, Box<dyn Error>> {
    let response = ureq::get("https://api.coincap.io/v2/assets?limit=100").call()?;

    let json: serde_json::Value = response.into_json()?;

    let data = json["data"].as_array().unwrap();

    data.iter()
        .map(|crypto| -> Result<CryptoPrice, Box<dyn Error>> {
            let name = crypto["name"].as_str().unwrap().to_string();
            let symbol = crypto["symbol"].as_str().unwrap().to_string();
            let price = crypto["priceUsd"].as_str().unwrap().parse()?;
            let change = crypto["changePercent24Hr"].as_str().unwrap().parse()?;
            let market_cap = crypto["marketCapUsd"].as_str().unwrap().parse()?;
            let volume_24h = crypto["volumeUsd24Hr"].as_str().unwrap().parse()?;

            Ok(CryptoPrice {
                name,
                symbol,
                price,
                change,
                market_cap,
                volume_24h,
            })
        })
        .collect()
}
