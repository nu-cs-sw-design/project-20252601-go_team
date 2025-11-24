// src/main.rs


use actix_web::{web, App, HttpServer, Responder, HttpResponse};
mod model;
use std::sync::Arc;
use rust_decimal::Decimal;
use model::{Side, TimeInForce, TradableAsset, Stock, LimitOrder, MarketOrder, Order, PriceLevel};

async fn hello() -> impl Responder {
    "Hello, world!"
}

async fn print_content(body: String) -> impl Responder {
    println!("Received content: {}", body);
    HttpResponse::Ok().body("Content printed to console")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let aapl = Arc::new(Stock::new("AAPL", "Apple Inc.", "Big fruit company")) as Arc<dyn TradableAsset>;

    // Create a price level at 100
    let mut lvl = PriceLevel::new(Decimal::new(100, 0)); // 100

    // Maker: SELL 10 @ 100
    let maker_order = LimitOrder::new(
        "ask1".into(),
        aapl.clone(),
        Side::SELL,
        Decimal::new(10, 0),
        0,
        TimeInForce::GTC,
        Decimal::new(100, 0),
    );
    lvl.add_order(maker_order);

    // Taker: BUY 6 @ any price (market order)
    let mut taker = MarketOrder::new(
        "m1".into(),
        aapl.clone(),
        Side::BUY,
        Decimal::new(6, 0),
        1,
        TimeInForce::IOC,
    );

    let trades = lvl.match_with(&mut taker);

    println!("Trades: {:#?}", trades);
    println!("Level volume: {}", lvl.total_volume());
    println!("Taker remaining: {}", taker.remaining_quantity());

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(hello))
            .route("/print", web::post().to(print_content))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

