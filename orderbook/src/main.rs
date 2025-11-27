// src/main.rs

// src/main.rs

mod model;

use std::sync::Arc;
use rust_decimal::Decimal;

use model::{
    Side,
    TimeInForce,
    TradableAsset,
    Stock,
    LimitOrder,
    MarketOrder,
    OrderBook,
    OrderBookObserver,
    OrderBookSnapshot,
    Trade,
    TopOfBook,
};

struct LoggingObserver;

impl OrderBookObserver for LoggingObserver {
    fn on_order_book_update(&self, snapshot: &OrderBookSnapshot) {
        println!(
            "[Observer] Book update: {} bid levels / {} ask levels",
            snapshot.bids.len(),
            snapshot.asks.len()
        );
    }

    fn on_new_trade(&self, trade: &Trade) {
        println!(
            "[Observer] New trade: {} @ {} (taker={}, maker={})",
            trade.quantity, trade.price, trade.taker_order_id, trade.maker_order_id
        );
    }
}

fn print_top_of_book(book: &OrderBook) {
    match book.get_top_of_book() {
        None => println!("Top of book: EMPTY"),
        Some(TopOfBook { best_bid, best_ask, spread }) => {
            println!("Top of book:");
            match best_bid {
                Some(bid) => println!("  Best Bid: {} @ {}", bid.total_volume, bid.price),
                None => println!("  Best Bid: None"),
            }
            match best_ask {
                Some(ask) => println!("  Best Ask: {} @ {}", ask.total_volume, ask.price),
                None => println!("  Best Ask: None"),
            }
            match spread {
                Some(s) => println!("  Spread: {}", s),
                None => println!("  Spread: N/A"),
            }
        }
    }
}

fn main() {
    // Create an asset (AAPL stock) and wrap it in Arc<dyn TradableAsset>
    let aapl: Arc<dyn TradableAsset> =
        Arc::new(Stock::new("AAPL", "Apple Inc.", "Big fruit company"));

    // Create an OrderBook and attach an observer
    let mut book = OrderBook::new(aapl.clone());
    book.add_observer(Arc::new(LoggingObserver));

    // ---------- 1. Add resting SELL and BUY limit orders ----------

    // Resting ask: SELL 10 @ 100 GTC
    let ask1 = LimitOrder::new(
        "ask1".into(),
        aapl.clone(),
        Side::SELL,
        Decimal::new(10, 0),     // 10
        0,                       // timestamp
        TimeInForce::GTC,
        Decimal::new(100, 0),    // price 100
    );
    let trades = book.add_limit_order(ask1);
    println!("Trades from adding ask1: {:?}", trades);
    print_top_of_book(&book);

    // Resting bid: BUY 5 @ 99 GTC
    let bid1 = LimitOrder::new(
        "bid1".into(),
        aapl.clone(),
        Side::BUY,
        Decimal::new(5, 0),      // 5
        1,
        TimeInForce::GTC,
        Decimal::new(99, 0),     // price 99
    );
    let trades = book.add_limit_order(bid1);
    println!("Trades from adding bid1: {:?}", trades);
    print_top_of_book(&book);

    // ---------- 2. Add a market BUY to hit the ask side ----------

    let mkt_buy = MarketOrder::new(
        "mkt1".into(),
        aapl.clone(),
        Side::BUY,
        Decimal::new(6, 0),      // 6
        2,
        TimeInForce::IOC,
    );
    let trades = book.add_market_order(mkt_buy);
    println!("Trades from market BUY mkt1: {:#?}", trades);
    print_top_of_book(&book);

    // ---------- 3. Modify remaining ask1 (if it still exists) ----------

    let modified = book.modify_order(
        "ask1",
        Decimal::new(101, 0),    // move ask up to 101
        Decimal::new(3, 0),      // set new qty = 3
    );
    println!("Modified ask1? {modified}");
    print_top_of_book(&book);

    // ---------- 4. Cancel bid1 ----------

    let cancelled = book.cancel_order("bid1");
    println!("Cancelled bid1? {cancelled}");
    print_top_of_book(&book);

    // ---------- 5. Inspect open orders and recent trades ----------

    let open_buys = book.get_open_orders(Side::BUY);
    let open_sells = book.get_open_orders(Side::SELL);
    println!("Open BUY orders: {}", open_buys.len());
    println!("Open SELL orders: {}", open_sells.len());

    let recent_trades = book.get_recent_trades(10);
    println!("Recent trades (up to 10): {:#?}", recent_trades);

    // ---------- 6. Full snapshot ----------

    let snapshot = book.get_snapshot();
    println!(
        "Snapshot: {} bid levels, {} ask levels, {} recent trades",
        snapshot.bids.len(),
        snapshot.asks.len(),
        snapshot.recent_trades.len()
    );
}


























//
// OLD MAIN:
//




// #[cfg(test)]
// #[path = "tests/api_test.rs"]
// mod api_test;

// pub mod controller;

// use actix_web::{web, App, HttpServer };
// mod model;

// use controller::controller::{
//     place_limit_order, place_market_order, cancel_order, modify_order,
//     get_order, get_open_orders, reset_book, get_recent_trades,
//     export_trades, export_book_history, health
// };

// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     HttpServer::new(|| {
//         App::new()
//             .route("/health", web::post().to(health))
//             .route("/place_limit_order", web::post().to(place_limit_order))
//             .route("/place_market_order", web::post().to(place_market_order))
//             .route("/cancel_order", web::post().to(cancel_order))
//             .route("/modify_order", web::post().to(modify_order))
//             .route("/get_order", web::get().to(get_order))
//             .route("/get_open_orders", web::get().to(get_open_orders))
//             .route("/reset_book", web::post().to(reset_book))
//             .route("/get_recent_trades", web::get().to(get_recent_trades))
//             .route("/export_trades", web::post().to(export_trades))
//             .route("/export_book_history", web::post().to(export_book_history))
//     })
//     .bind("127.0.0.1:8080")?
//     .run()
//     .await
// }

