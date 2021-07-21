extern crate serde;
extern crate serde_json;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize)]
struct DayPrice {
    price: f64,
    timestamp: u64,
}

#[derive(Serialize, Deserialize)]
struct Ohlc {
    high: String,
    timestamp: String,
    volume: String,
    low: String,
    close: String,
    open: String,
}

#[derive(Serialize, Deserialize)]
struct OhlcData {
    pair: String,
    ohlc: Vec<Ohlc>,
}

#[derive(Serialize, Deserialize)]
struct BitstampPriceData {
    data: OhlcData,
}

fn main() {
    let mut daily_prices: Vec<DayPrice> = Vec::new();

    // Read in price files
    for n in 2015..2020 {
        // Read the data
        let buff = fs::read_to_string(format!("./prices/{}.json", n)).unwrap();

        // Parse the json
        let parsed_json: BitstampPriceData = serde_json::from_str(&buff).unwrap();

        // Iterate over all the prices
        for ohlc in parsed_json.data.ohlc.iter() {
            // Convert and add to the list
            let timestamp: u64 = ohlc.timestamp.parse().unwrap();
            let price: f64 = ohlc.open.parse().unwrap();
            daily_prices.push(DayPrice { price, timestamp })
        }
    }

    // Sort the prices by day timestamp just in case
    daily_prices.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    for daily in daily_prices.iter() {
        println!("{} {}", daily.timestamp, daily.price)
    }
}
