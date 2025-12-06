use actix_web::{test, web, App};
use orderbook::controller::controller::{
    cancel_order, modify_order, place_limit_order, place_market_order, PlaceLimitOrderResponse,
    PlaceMarketOrderResponse,
};
use rust_decimal::Decimal;
use serde_json::json;

#[actix_web::test]
async fn modify_order_updates_price_and_quantity() {
    let app = test::init_service(
        App::new()
            .route("/place_limit_order", web::post().to(place_limit_order))
            .route("/modify_order", web::post().to(modify_order))
            .route("/place_market_order", web::post().to(place_market_order))
            .route("/cancel_order", web::post().to(cancel_order)),
    )
    .await;

    // Seed a SELL order that we will adjust.
    let create_req = test::TestRequest::post()
        .uri("/place_limit_order")
        .set_json(json!({
            "symbol": "MOD-SUCCESS",
            "side": "SELL",
            "price": "100.00",
            "quantity": "10",
            "tif": "GTC"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert!(create_resp.status().is_success());

    let place_payload: PlaceLimitOrderResponse = test::read_body_json(create_resp).await;
    let order_id = place_payload.order_id;

    // Modify order to reduce quantity and move price higher.
    let modify_req = test::TestRequest::post()
        .uri("/modify_order")
        .set_json(json!({
            "symbol": "MOD-SUCCESS",
            "order_id": order_id,
            "new_price": "105.50",
            "new_quantity": "6"
        }))
        .to_request();

    let modify_resp = test::call_service(&app, modify_req).await;
    assert!(modify_resp.status().is_success());

    let modified: bool = test::read_body_json(modify_resp).await;
    assert!(modified);

    // Fill against the modified order and verify trade is generated at new terms.
    let taker_req = test::TestRequest::post()
        .uri("/place_market_order")
        .set_json(json!({
            "symbol": "MOD-SUCCESS",
            "side": "BUY",
            "quantity": "6",
            "tif": "IOC"
        }))
        .to_request();

    let taker_resp = test::call_service(&app, taker_req).await;
    assert!(taker_resp.status().is_success());

    let market_payload: PlaceMarketOrderResponse = test::read_body_json(taker_resp).await;
    assert_eq!(market_payload.executed_quantity, Decimal::new(6, 0));
    assert!(market_payload.fully_filled);
    assert!(market_payload.trades.iter().any(|trade| {
        trade.quantity == Decimal::new(6, 0) && trade.price == Decimal::new(10550, 2)
    }));
}

#[actix_web::test]
async fn modify_order_with_invalid_values_rejected() {
    let app =
        test::init_service(App::new().route("/modify_order", web::post().to(modify_order))).await;

    let bad_price_req = test::TestRequest::post()
        .uri("/modify_order")
        .set_json(json!({
            "symbol": "MOD-INVALID",
            "order_id": "ord-1",
            "new_price": "0",
            "new_quantity": "5"
        }))
        .to_request();

    let bad_price_resp = test::call_service(&app, bad_price_req).await;
    assert_eq!(
        bad_price_resp.status(),
        actix_web::http::StatusCode::BAD_REQUEST
    );

    let bad_qty_req = test::TestRequest::post()
        .uri("/modify_order")
        .set_json(json!({
            "symbol": "MOD-INVALID",
            "order_id": "ord-1",
            "new_price": "10",
            "new_quantity": "0"
        }))
        .to_request();

    let bad_qty_resp = test::call_service(&app, bad_qty_req).await;
    assert_eq!(
        bad_qty_resp.status(),
        actix_web::http::StatusCode::BAD_REQUEST
    );
}

#[actix_web::test]
async fn modify_order_unknown_id_returns_false() {
    let app = test::init_service(
        App::new()
            .route("/place_limit_order", web::post().to(place_limit_order))
            .route("/modify_order", web::post().to(modify_order)),
    )
    .await;

    let create_req = test::TestRequest::post()
        .uri("/place_limit_order")
        .set_json(json!({
            "symbol": "MOD-UNKNOWN",
            "side": "BUY",
            "price": "20.00",
            "quantity": "1",
            "tif": "GTC"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert!(create_resp.status().is_success());

    let hidden_id = "ord-nonexistent";

    let modify_req = test::TestRequest::post()
        .uri("/modify_order")
        .set_json(json!({
            "symbol": "MOD-UNKNOWN",
            "order_id": hidden_id,
            "new_price": "25.00",
            "new_quantity": "1"
        }))
        .to_request();

    let modify_resp = test::call_service(&app, modify_req).await;
    assert!(modify_resp.status().is_success());

    let result: bool = test::read_body_json(modify_resp).await;
    assert!(!result);
}
