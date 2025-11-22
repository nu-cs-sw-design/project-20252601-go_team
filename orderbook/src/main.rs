// #![allow(dead_code)]

mod model;
use model::{Side, TimeInForce, TradableAsset, Stock, Crypto};

fn main() {
    let aapl = Stock::new("AAPL", "Apple Inc.", "Big fruit company");
    println!("Stock Ticker: {}", aapl.ticker());
    println!("Stock Name: {}", aapl.name());
    println!("Stock Description: {}", aapl.description());

    let btc = Crypto::new("BTC", "Bitcoin", "Digital gold");
    println!("Crypto Ticker: {}", btc.ticker());
    println!("Crypto Name: {}", btc.name());
    println!("Crypto Description: {}", btc.description());

    let buy_side = Side::BUY;
    let order_tif = TimeInForce::GTC;

    println!("Example Side: {:?}", buy_side);
    println!("Example TimeInForce: {:?}", order_tif);
}