// src/main.rs

#[cfg(test)]
#[path = "tests/api_test.rs"]
mod api_test;

pub mod controller;

use actix_web::{web, App, HttpServer };
mod model;

use controller::controller::{
    place_limit_order, place_market_order, cancel_order, modify_order,
    get_order, get_open_orders, reset_book, get_recent_trades,
    export_trades, export_book_history, health
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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

