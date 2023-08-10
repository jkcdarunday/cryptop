extern crate serde_json;
extern crate ureq;

pub struct CryptoPrice {
    pub name: String,
    pub symbol: String,
    pub price: f64,
    pub change: f64,
    pub market_cap: f64,
    pub volume_24h: f64,
}

pub fn get_top_cryptos() -> Vec<CryptoPrice> {
    let response = ureq::get("https://api.coincap.io/v2/assets?limit=100")
        .call()
        .unwrap();

    let json: serde_json::Value = response.into_json().unwrap();

    let data = json["data"].as_array().unwrap();

    data.iter()
        .map(|crypto| {
            let name = crypto["name"].as_str().unwrap().to_string();
            let symbol = crypto["symbol"].as_str().unwrap().to_string();
            let price = crypto["priceUsd"].as_str().unwrap().parse().unwrap();
            let change = crypto["changePercent24Hr"].as_str().unwrap().parse().unwrap();
            let market_cap = crypto["marketCapUsd"].as_str().unwrap().parse().unwrap();
            let volume_24h = crypto["volumeUsd24Hr"].as_str().unwrap().parse().unwrap();

            CryptoPrice {
                name,
                symbol,
                price,
                change,
                market_cap,
                volume_24h,
            }
        })
        .collect()
}
