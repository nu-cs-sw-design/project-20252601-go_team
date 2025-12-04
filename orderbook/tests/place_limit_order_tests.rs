use actix_web::{test, web, App};
use orderbook::controller::controller::{place_limit_order, OrderDto, PlaceLimitOrderResponse};
use rust_decimal::Decimal;
use serde_json::json;

#[actix_web::test]
async fn place_limit_order_creates_resting_order() {
    let app = test::init_service(
        App::new().route("/place_limit_order", web::post().to(place_limit_order)),
    )
    .await;

    let request = test::TestRequest::post()
        .uri("/place_limit_order")
        .set_json(json!({
            "symbol": "TEST-REST",
            "side": "BUY",
            "price": "150.50",
            "quantity": "10",
            "tif": "GTC"
        }))
        .to_request();

    let response = test::call_service(&app, request).await;
    assert!(response.status().is_success());

    let body = test::read_body(response).await;
    let payload: PlaceLimitOrderResponse = serde_json::from_slice(&body).unwrap();

    assert!(payload.order_id.starts_with("ord-TEST-REST-"));
    assert!(payload.trades.is_empty());

    let resting: OrderDto = payload.resting_order.expect("expected resting order");
    assert_eq!(resting.symbol, "TEST-REST");
    assert_eq!(resting.side, "BUY");
    assert_eq!(resting.price, Some(Decimal::new(15050, 2)));
    assert_eq!(resting.remaining_quantity, Decimal::new(10, 0));
    assert_eq!(resting.tif, "GTC");
}

#[actix_web::test]
async fn place_limit_order_crosses_existing_liquidity() {
    let app = test::init_service(
        App::new().route("/place_limit_order", web::post().to(place_limit_order)),
    )
    .await;

    // Seed the book with a resting SELL order
    let seed_request = test::TestRequest::post()
        .uri("/place_limit_order")
        .set_json(json!({
            "symbol": "TEST-CROSS",
            "side": "SELL",
            "price": "100.00",
            "quantity": "5",
            "tif": "GTC"
        }))
        .to_request();

    let seed_response = test::call_service(&app, seed_request).await;
    assert!(seed_response.status().is_success());

    // Submit a BUY order that should cross and fully fill the resting SELL
    let taker_request = test::TestRequest::post()
        .uri("/place_limit_order")
        .set_json(json!({
            "symbol": "TEST-CROSS",
            "side": "BUY",
            "price": "101.00",
            "quantity": "5",
            "tif": "IOC"
        }))
        .to_request();

    let taker_response = test::call_service(&app, taker_request).await;
    assert!(taker_response.status().is_success());

    let body = test::read_body(taker_response).await;
    let payload: PlaceLimitOrderResponse = serde_json::from_slice(&body).unwrap();

    assert!(payload.trades.len() >= 1);
    assert!(payload.resting_order.is_none());

    let executed = &payload.trades[0];
    assert_eq!(executed.symbol, "TEST-CROSS");
    assert_eq!(executed.quantity, Decimal::new(5, 0));
    assert!(executed.maker_order_id.starts_with("ord-TEST-CROSS-"));
    assert_eq!(executed.taker_order_id, payload.order_id);
    assert_eq!(executed.price, Decimal::new(10000, 2));
}

#[actix_web::test]
async fn place_limit_order_fok_rejects_without_liquidity() {
    let app = test::init_service(
        App::new().route("/place_limit_order", web::post().to(place_limit_order)),
    )
    .await;

    let fok_request = test::TestRequest::post()
        .uri("/place_limit_order")
        .set_json(json!({
            "symbol": "TEST-FOK",
            "side": "BUY",
            "price": "100.00",
            "quantity": "3",
            "tif": "FOK"
        }))
        .to_request();

    let fok_response = test::call_service(&app, fok_request).await;
    assert!(fok_response.status().is_success());

    let body = test::read_body(fok_response).await;
    let payload: PlaceLimitOrderResponse = serde_json::from_slice(&body).unwrap();

    assert!(payload.trades.is_empty());
    assert!(payload.resting_order.is_none());
}

#[actix_web::test]
async fn place_limit_order_rejects_invalid_side() {
    let app = test::init_service(
        App::new().route("/place_limit_order", web::post().to(place_limit_order)),
    )
    .await;

    let bad_request = test::TestRequest::post()
        .uri("/place_limit_order")
        .set_json(json!({
            "symbol": "TEST-BAD",
            "side": "HOLD",
            "price": "20.00",
            "quantity": "1",
            "tif": "GTC"
        }))
        .to_request();

    let response = test::call_service(&app, bad_request).await;
    assert_eq!(response.status(), actix_web::http::StatusCode::BAD_REQUEST);
}
