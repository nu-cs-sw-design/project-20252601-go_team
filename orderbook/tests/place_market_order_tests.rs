use actix_web::{test, web, App};
use orderbook::controller::controller::{
    place_limit_order, place_market_order, PlaceMarketOrderResponse,
};
use rust_decimal::Decimal;
use serde_json::json;
use std::collections::HashSet;

#[actix_web::test]
async fn market_order_executes_available_liquidity() {
    let app = test::init_service(
        App::new()
            .route("/place_limit_order", web::post().to(place_limit_order))
            .route("/place_market_order", web::post().to(place_market_order)),
    )
    .await;

    // Seed the book with a resting sell order.
    let seed = test::TestRequest::post()
        .uri("/place_limit_order")
        .set_json(json!({
            "symbol": "MKT-IOC-EXEC",
            "side": "SELL",
            "price": "100.00",
            "quantity": "5",
            "tif": "GTC"
        }))
        .to_request();
    let resp = test::call_service(&app, seed).await;
    assert!(resp.status().is_success());

    // Submit a market BUY that should fully fill against the resting sell order.
    let taker = test::TestRequest::post()
        .uri("/place_market_order")
        .set_json(json!({
            "symbol": "MKT-IOC-EXEC",
            "side": "BUY",
            "quantity": "5",
            "tif": "IOC"
        }))
        .to_request();

    let resp = test::call_service(&app, taker).await;
    assert!(resp.status().is_success());

    let payload: PlaceMarketOrderResponse = test::read_body_json(resp).await;
    assert!(payload.trades.len() >= 1);
    assert_eq!(payload.executed_quantity, Decimal::new(5, 0));
    assert!(payload.fully_filled);
    assert_eq!(payload.tif, "IOC");
}

#[actix_web::test]
async fn market_order_partial_fill_ioc() {
    let app = test::init_service(
        App::new()
            .route("/place_limit_order", web::post().to(place_limit_order))
            .route("/place_market_order", web::post().to(place_market_order)),
    )
    .await;

    let seed = test::TestRequest::post()
        .uri("/place_limit_order")
        .set_json(json!({
            "symbol": "MKT-IOC-PART",
            "side": "SELL",
            "price": "50.00",
            "quantity": "3",
            "tif": "GTC"
        }))
        .to_request();
    let resp = test::call_service(&app, seed).await;
    assert!(resp.status().is_success());

    let taker = test::TestRequest::post()
        .uri("/place_market_order")
        .set_json(json!({
            "symbol": "MKT-IOC-PART",
            "side": "BUY",
            "quantity": "5",
            "tif": "IOC"
        }))
        .to_request();

    let resp = test::call_service(&app, taker).await;
    assert!(resp.status().is_success());

    let payload: PlaceMarketOrderResponse = test::read_body_json(resp).await;
    assert_eq!(payload.executed_quantity, Decimal::new(3, 0));
    assert!(!payload.fully_filled);
    assert!(payload.trades.len() >= 1);
}

#[actix_web::test]
async fn market_order_fok_rejects_without_liquidity() {
    let app = test::init_service(
        App::new().route("/place_market_order", web::post().to(place_market_order)),
    )
    .await;

    let taker = test::TestRequest::post()
        .uri("/place_market_order")
        .set_json(json!({
            "symbol": "MKT-FOK-NOLIQ",
            "side": "BUY",
            "quantity": "2",
            "tif": "FOK"
        }))
        .to_request();

    let resp = test::call_service(&app, taker).await;
    assert!(resp.status().is_success());

    let payload: PlaceMarketOrderResponse = test::read_body_json(resp).await;
    assert_eq!(payload.executed_quantity, Decimal::new(0, 0));
    assert!(payload.trades.is_empty());
    assert!(!payload.fully_filled);
}

#[actix_web::test]
async fn market_order_rejects_empty_symbol() {
    let app = test::init_service(
        App::new().route("/place_market_order", web::post().to(place_market_order)),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/place_market_order")
        .set_json(json!({
            "symbol": " ",
            "side": "BUY",
            "quantity": "1",
            "tif": "IOC"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn market_orders_generate_unique_order_ids() {
    let app = test::init_service(
        App::new().route("/place_market_order", web::post().to(place_market_order)),
    )
    .await;

    let mut ids = HashSet::new();

    for _ in 0..2 {
        let req = test::TestRequest::post()
            .uri("/place_market_order")
            .set_json(json!({
                "symbol": "MKT-UNIQ",
                "side": "BUY",
                "quantity": "1",
                "tif": "IOC"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let payload: PlaceMarketOrderResponse = test::read_body_json(resp).await;
        ids.insert(payload.order_id);
    }

    assert_eq!(
        ids.len(),
        2,
        "expected unique order IDs for rapid submissions"
    );
}
