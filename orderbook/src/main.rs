// src/main.rs

mod model;

use rust_decimal::Decimal;
use std::sync::Arc;

use model::{
    LimitOrder, MarketOrder, Order, OrderBook, OrderBookObserver, OrderBookSnapshot, Side, Stock,
    TimeInForce, TradableAsset, Trade,
};
#[cfg(test)]
#[path = "tests/api_test.rs"]
mod api_test;

pub mod controller;

use actix_web::{web, App, HttpServer };

use controller::controller::{
    place_limit_order, place_market_order, cancel_order, modify_order,
    get_order, get_open_orders, reset_book, get_recent_trades,
    export_trades, export_book_history, health
};

#[derive(Debug)]
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

fn print_top_of_book(book: &OrderBook, label: &str) {
    println!("--- Top of book ({label}) ---");
    if let Some(tob) = book.get_top_of_book() {
        match tob.best_bid {
            Some(bid) => println!("  Best Bid: {} @ {}", bid.total_volume, bid.price),
            None => println!("  Best Bid: None"),
        }
        match tob.best_ask {
            Some(ask) => println!("  Best Ask: {} @ {}", ask.total_volume, ask.price),
            None => println!("  Best Ask: None"),
        }
        match tob.spread {
            Some(spread) => println!("  Spread: {}", spread),
            None => println!("  Spread: N/A"),
        }
    } else {
        println!("  Book is empty.");
    }
    println!();
}

async fn run_server() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/health", web::post().to(health))
            .route("/place_limit_order", web::post().to(place_limit_order))
            .route("/place_market_order", web::post().to(place_market_order))
            .route("/cancel_order", web::post().to(cancel_order))
            .route("/modify_order", web::post().to(modify_order))
            .route("/get_order", web::get().to(get_order))
            .route("/get_open_orders", web::get().to(get_open_orders))
            .route("/reset_book", web::post().to(reset_book))
            .route("/get_recent_trades", web::get().to(get_recent_trades))
            .route("/export_trades", web::post().to(export_trades))
            .route("/export_book_history", web::post().to(export_book_history))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // ----- Setup asset + order book -----

    let aapl: Arc<dyn TradableAsset> =
        Arc::new(Stock::new("AAPL", "Apple Inc.", "Big fruit company"));

    let mut book = OrderBook::new(aapl.clone());
    book.add_observer(Arc::new(LoggingObserver));

    // ----- 1. Add an ask: SELL 10 @ 100 GTC -----

    let ask1 = LimitOrder::new(
        "ask1".to_string(),
        aapl.clone(),
        Side::SELL,
        Decimal::new(10, 0), // 10
        0,                   // timestamp
        TimeInForce::GTC,
        Decimal::new(100, 0), // price = 100
    );

    let trades_ask1 = book.add_limit_order(ask1);
    println!("Trades from adding ask1: {trades_ask1:?}");
    print_top_of_book(&book, "after ask1");

    // ----- 2. Add a bid: BUY 5 @ 99 GTC (no cross yet) -----

    let bid1 = LimitOrder::new(
        "bid1".to_string(),
        aapl.clone(),
        Side::BUY,
        Decimal::new(5, 0),
        1,
        TimeInForce::GTC,
        Decimal::new(99, 0),
    );

    let trades_bid1 = book.add_limit_order(bid1);
    println!("Trades from adding bid1: {trades_bid1:?}");
    print_top_of_book(&book, "after bid1");

    // ----- 3. Add market BUY: 6 IOC (should trade against ask1 at 100) -----

    let mkt_buy = MarketOrder::new(
        "mkt1".to_string(),
        aapl.clone(),
        Side::BUY,
        Decimal::new(6, 0),
        2,
        TimeInForce::IOC,
    );

    let trades_mkt = book.add_market_order(mkt_buy);
    println!("Trades from market BUY mkt1: {trades_mkt:#?}");
    print_top_of_book(&book, "after mkt1");

    // ----- 4. FOK limit BUY that should FAIL (not enough volume) -----
    //
    // After mkt1, ask1 had 10 - 6 = 4 remaining @ 100.
    // So trying to FOK buy 5 @ 100 should see only 4 available (<= 100) and
    // therefore produce NO trades.

    let fok_fail = LimitOrder::new(
        "fok_fail".to_string(),
        aapl.clone(),
        Side::BUY,
        Decimal::new(5, 0),
        3,
        TimeInForce::FOK,
        Decimal::new(100, 0),
    );

    let trades_fok_fail = book.add_limit_order(fok_fail);
    println!("Trades from FOK BUY fok_fail (expect none): {trades_fok_fail:?}");
    print_top_of_book(&book, "after fok_fail");

    // ----- 5. Add another ask: SELL 2 @ 101 GTC -----

    let ask2 = LimitOrder::new(
        "ask2".to_string(),
        aapl.clone(),
        Side::SELL,
        Decimal::new(2, 0),
        4,
        TimeInForce::GTC,
        Decimal::new(101, 0),
    );

    let trades_ask2 = book.add_limit_order(ask2);
    println!("Trades from adding ask2: {trades_ask2:?}");
    print_top_of_book(&book, "after ask2");

    // Now asks are:
    //   ask1: 4 @ 100
    //   ask2: 2 @ 101
    // Total available at <= 101 = 6

    // ----- 6. FOK limit BUY that should SUCCEED (enough volume) -----

    let fok_ok = LimitOrder::new(
        "fok_ok".to_string(),
        aapl.clone(),
        Side::BUY,
        Decimal::new(6, 0),
        5,
        TimeInForce::FOK,
        Decimal::new(101, 0),
    );

    let trades_fok_ok = book.add_limit_order(fok_ok);
    println!(
        "Trades from FOK BUY fok_ok (expect full fill in one or more trades):\
         \n{trades_fok_ok:#?}"
    );
    print_top_of_book(&book, "after fok_ok");

    // ----- 7. Inspect open orders, recent trades, snapshot -----

    let open_buys = book.get_open_orders(Side::BUY);
    let open_sells = book.get_open_orders(Side::SELL);

    println!("Open BUY orders: {}", open_buys.len());
    for o in open_buys {
        println!(
            "  BUY {} @ {} (id={})",
            o.remaining_quantity(),
            o.price(),
            o.order_id()
        );
    }

    println!("Open SELL orders: {}", open_sells.len());
    for o in open_sells {
        println!(
            "  SELL {} @ {} (id={})",
            o.remaining_quantity(),
            o.price(),
            o.order_id()
        );
    }

    let recent_trades = book.get_recent_trades(20);
    println!("Recent trades (up to 20): {:#?}", recent_trades);

    let snapshot = book.get_snapshot();
    println!(
        "Snapshot: {} bid levels, {} ask levels, {} recent trades",
        snapshot.bids.len(),
        snapshot.asks.len(),
        snapshot.recent_trades.len()
    );
    run_server().await
}