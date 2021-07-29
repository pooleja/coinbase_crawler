extern crate serde;
extern crate serde_json;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::fs;

struct DayPrice {
    price: f64,
    timestamp: i64,
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
// portfolio,type,time,amount,balance,amount/balance unit,transfer id,trade id,order id
#[derive(Debug, Deserialize)]
struct TradeRecord {
    portfolio: String,
    #[serde(rename = "type")]
    action: String,
    time: String,
    amount: f64,
    balance: f64,
    #[serde(rename = "amount/balance unit")]
    unit: String,
    #[serde(rename = "transfer id")]
    transfer_id: String,
    #[serde(rename = "trade id")]
    trade_id: String,
    #[serde(rename = "order id")]
    order_id: String,
}

#[derive(Debug)]
struct KnownLot {
    date: String,
    balance: f64,
    cost_basis: f64,
}

fn get_year_from_record(record: &TradeRecord) -> i32 {
    let rfc3339 = DateTime::parse_from_rfc3339(&record.time).unwrap();
    rfc3339.year()
}

// Not very efficient
fn get_price_for_date(date: &String, daily_prices: &Vec<DayPrice>) -> f64 {
    let rfc3339 = DateTime::parse_from_rfc3339(&date).unwrap();

    // Iterate over the prices from the beginning until a price earlier is found
    for price in daily_prices.iter() {
        if rfc3339.timestamp() < price.timestamp {
            return price.price;
        }
    }

    let last = daily_prices.last().unwrap();
    println!("{} {} {}", date, last.timestamp, rfc3339.timestamp());

    panic!("Price not found");
}

fn round_to_8(num: f64) -> f64 {
    (num * 100000000_f64).round() / 100000000_f64
}

fn remove_known_lots_for_amt(amount: f64, known_lots: &mut VecDeque<KnownLot>) -> Vec<KnownLot> {
    // println!("looking for lots that add up to {}", amount);
    // println!("{:?}", known_lots);
    let mut found_lots: Vec<KnownLot> = Vec::new();
    let mut total_of_lots = 0_f64;

    loop {
        if known_lots.len() == 0 {
            println!(
                "Ran out of lots due to f_64 rounding!!! Expected at least {}, Using cost basis of this amount {}",
                amount, total_of_lots
            );
            break;
        }

        let lot = known_lots.pop_front().unwrap();
        total_of_lots = round_to_8(total_of_lots + lot.balance);

        // println!("{} {} ", total_of_lots, amount);

        // 3 scenarios
        if total_of_lots == amount {
            // First - magically equal
            found_lots.push(lot);
            break;
        } else if total_of_lots < amount {
            // Second - not enough added up yet
            found_lots.push(lot);
        } else {
            // Third - last lot put us over - and round to 8 digits
            let extra_amount = round_to_8(total_of_lots - amount);

            // Get the percent of the lot amount being used
            let lot_percentage = round_to_8(lot.balance - extra_amount) / lot.balance;

            // Figure out how much of the cost basis is being used
            let cost_basis_used = round_to_8(lot.cost_basis * lot_percentage);

            // Push the leftover amt to the front of the known lots list
            known_lots.push_front(KnownLot {
                balance: extra_amount,
                date: lot.date.clone(),
                cost_basis: lot.cost_basis - cost_basis_used,
            });

            // Update the last lot amt and put it into the list
            found_lots.push(KnownLot {
                balance: lot.balance - extra_amount,
                date: lot.date.clone(),
                cost_basis: cost_basis_used,
            });
            break;
        }
    }

    return found_lots;
}

#[derive(Debug, Deserialize)]
struct CostOverride {
    deposit: String,
    cost_basis: f64,
    amount: f64,
    date: String,
}

fn get_override_known_lot(transfer_id: String) -> Option<KnownLot> {
    // Check if the override file exists
    if fs::metadata("./deposits/deposits.csv").is_ok() {
        let mut rdr = csv::Reader::from_path("./deposits/deposits.csv").unwrap();
        let mut record_itr: csv::DeserializeRecordsIter<'_, std::fs::File, CostOverride> =
            rdr.deserialize();

        while let Some(item) = record_itr.next() {
            let deposit = item.unwrap();
            if deposit.deposit == transfer_id {
                return Some(KnownLot {
                    date: deposit.date,
                    balance: deposit.amount,
                    cost_basis: deposit.cost_basis,
                });
            }
        }
    }

    return None;
}

#[derive(Debug, Serialize)]
struct YearlySummary {
    fees: f64,
    rebates: f64,
    short_term_gains: f64,
    long_term_gains: f64,
    total_sales: f64,
    total_buys: f64,
}

fn main() {
    let mut daily_prices: Vec<DayPrice> = Vec::new();
    let mut known_lots: VecDeque<KnownLot> = VecDeque::new();
    let mut yearly_summary: HashMap<i32, YearlySummary> = HashMap::new();

    // Read in price files and initialize vectors
    for n in 2015..=2020 {
        // Read the data
        let buff = fs::read_to_string(format!("./prices/{}.json", n)).unwrap();

        // Parse the json
        let parsed_json: BitstampPriceData = serde_json::from_str(&buff).unwrap();

        // Iterate over all the prices
        for ohlc in parsed_json.data.ohlc.iter() {
            // Convert and add to the list
            let timestamp: i64 = ohlc.timestamp.parse().unwrap();
            let price: f64 = ohlc.open.parse().unwrap();
            daily_prices.push(DayPrice { price, timestamp })
        }

        // Init to 0
        yearly_summary.insert(
            n,
            YearlySummary {
                fees: 0_f64,
                rebates: 0_f64,
                short_term_gains: 0_f64,
                long_term_gains: 0_f64,
                total_sales: 0_f64,
                total_buys: 0_f64,
            },
        );
    }

    // Sort the prices by day timestamp just in case
    daily_prices.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    // for daily in daily_prices.iter() {
    //     println!("{} {}", daily.timestamp, daily.price)
    // }

    // Get the CSV data
    let mut rdr = csv::Reader::from_path("./trades/trades.csv").unwrap();
    let mut record_itr: csv::DeserializeRecordsIter<'_, std::fs::File, TradeRecord> =
        rdr.deserialize();

    let mut found_year = 2015;

    // Create the writer for the output
    let mut wtr = csv::Writer::from_path(std::format!("./output/{}.csv", found_year)).unwrap();
    wtr.write_record(&[
        "Date",
        "Trade ID",
        "Order ID",
        "Crypto Sold",
        "USD Value Sold",
        "Crypto Acquire Date",
        "Cost Basis",
        "Long Term Gains/Loss",
        "Short Term Gains/Loss",
    ])
    .unwrap();

    // Iterate over the rows...
    while let Some(item) = record_itr.next() {
        let record = item.unwrap();
        // println!("{:?}", record);

        // Get the record year
        let year = get_year_from_record(&record);

        if found_year != year {
            found_year = year;
            // println!("beginning of {} {:?}", found_year, known_lots);

            // Start a new output file for a new year
            wtr.flush().unwrap();
            wtr = csv::Writer::from_path(std::format!("./output/{}.csv", found_year)).unwrap();
            wtr.write_record(&[
                "Date",
                "Trade ID",
                "Order ID",
                "Crypto Sold",
                "USD Value Sold",
                "Crypto Acquire Date",
                "Cost Basis",
                "Long Term Gains/Loss",
                "Short Term Gains/Loss",
            ])
            .unwrap();
        }

        match record.action.as_str() {
            "deposit" => {
                // Skip anything but BTC
                if record.unit != "BTC" {
                    continue;
                }

                // Check for override
                let known_lot_override = get_override_known_lot(record.transfer_id);

                match known_lot_override {
                    // Override found
                    Some(basis) => {
                        // println!("Deposit cost basis override {:?}", basis);

                        known_lots.push_back(basis);
                    }
                    // No override, use the daily price
                    None => {
                        let price = get_price_for_date(&record.time, &daily_prices);

                        let lot = KnownLot {
                            balance: record.amount,
                            date: record.time,
                            cost_basis: price * record.amount,
                        };
                        // println!("Deposit {:?}", lot);
                        // Add deposit to known lots
                        known_lots.push_back(lot);
                    }
                }
            }
            "match" => {
                // Skip anything but BTC
                if record.unit != "BTC" {
                    continue;
                }

                // Check if buy or sell
                if record.amount < 0_f64 {
                    // Get the next row to know how much USD was gained from the sale
                    let next_item = record_itr.next().unwrap().unwrap();
                    let usd_earned = next_item.amount;

                    // println!("Before sell");
                    // println!("{:?}", known_lots);

                    // Sell amounts are negative
                    let raw_amount = record.amount * -1_f64;

                    // Sells remove known lots
                    let lots_used = remove_known_lots_for_amt(raw_amount, &mut known_lots);

                    // println!("After sell");
                    // println!("{:?}", known_lots);

                    // println!("Lots found");
                    // println!("{:?}", lots_used);

                    for lot in lots_used {
                        // Get the percentage of the sale that this lot represents
                        let lot_percent = lot.balance / raw_amount;

                        // Get the USD amount that this lot earned from the sale
                        let lot_usd_earned = usd_earned * lot_percent;

                        // Get the amount of crypto that was sold
                        let crypto_sold = raw_amount * lot_percent;

                        // Get the total gain or loss - if earned more that cost basis, it is a gain... negative is a loss
                        let gain_or_loss = lot_usd_earned - lot.cost_basis;

                        // Check the difference in the dates - over 1 year is long term... less is short term
                        let record_date = DateTime::parse_from_rfc3339(&record.time).unwrap();
                        let lot_date = DateTime::parse_from_rfc3339(&lot.date).unwrap();
                        let seconds_in_a_year = 31536000;

                        let mut short_term = 0_f64;
                        let mut long_term = 0_f64;

                        if record_date.timestamp() - lot_date.timestamp() > seconds_in_a_year {
                            // Long term
                            let summary = yearly_summary.get_mut(&year).unwrap();
                            summary.long_term_gains += gain_or_loss;
                            summary.total_sales += lot_usd_earned;
                            long_term = gain_or_loss;
                        } else {
                            // Short term
                            let summary = yearly_summary.get_mut(&year).unwrap();
                            summary.short_term_gains += gain_or_loss;
                            summary.total_sales += lot_usd_earned;
                            short_term = gain_or_loss;
                        }

                        wtr.write_record(&[
                            record.time.clone(),
                            record.trade_id.clone(),
                            record.transfer_id.clone(),
                            record.order_id.clone(),
                            crypto_sold.to_string(),
                            lot_usd_earned.to_string(),
                            lot.date,
                            lot.cost_basis.to_string(),
                            long_term.to_string(),
                            short_term.to_string(),
                        ])
                        .unwrap();
                    }
                } else {
                    // Buys add to known lots
                    // Get the next match to know how much was paid for it (USD value is negative on buys)
                    let next_item = record_itr.next().unwrap().unwrap();
                    let lot = KnownLot {
                        balance: record.amount,
                        date: record.time,
                        cost_basis: next_item.amount * -1_f64,
                    };
                    // println!("Buy - {:?}", lot);
                    known_lots.push_back(lot);

                    let summary = yearly_summary.get_mut(&year).unwrap();
                    summary.total_buys += next_item.amount * -1_f64
                }
            }
            "withdrawal" => {
                // Skip anything but BTC
                if record.unit != "BTC" {
                    continue;
                }

                // Withdraw amounts are negative
                let raw_amount = record.amount * -1_f64;

                // Remove oldest known lots
                remove_known_lots_for_amt(raw_amount, &mut known_lots);
            }
            "fee" => {
                // Check what unit the fee is in
                if record.unit == "USD" {
                    // Save off the fee
                    let summary = yearly_summary.get_mut(&year).unwrap();
                    summary.fees += record.amount;
                } else if record.unit == "BTC" {
                    // Calculate the USD amount
                    let price = get_price_for_date(&record.time, &daily_prices);
                    let summary = yearly_summary.get_mut(&year).unwrap();
                    summary.fees += price * record.amount;
                }
            }
            "rebate" => {
                // Check what unit the rebate is in
                if record.unit == "USD" {
                    // Save off the rebate
                    let summary = yearly_summary.get_mut(&year).unwrap();
                    summary.rebates += record.amount;
                } else if record.unit == "BTC" {
                    // Calculate the USD amount
                    let price = get_price_for_date(&record.time, &daily_prices);
                    let summary = yearly_summary.get_mut(&year).unwrap();
                    summary.rebates += price * record.amount;
                }
            }
            // Ignore conversions as they are not taxable
            "conversion" => continue,
            _ => {
                println!("{:?}", record.action);
                panic!("Unknown Action")
            }
        }
    }

    wtr.flush().unwrap();

    // Write out yearly summary
    let mut summary_writer = csv::Writer::from_path("./output/summary.csv").unwrap();

    summary_writer
        .write_record(&[
            "Year",
            "Fees",
            "Rebates",
            "Short Term Gains",
            "Long Term Gains",
            "Total Sales",
            "Total Buys",
        ])
        .unwrap();

    for (key, value) in &yearly_summary {
        summary_writer
            .write_record(&[
                key.to_string(),
                value.fees.to_string(),
                value.rebates.to_string(),
                value.short_term_gains.to_string(),
                value.long_term_gains.to_string(),
                value.total_sales.to_string(),
                value.total_buys.to_string(),
            ])
            .unwrap();
    }

    summary_writer.flush().unwrap();

    // println!("Ending lots {:?}", known_lots);
}
