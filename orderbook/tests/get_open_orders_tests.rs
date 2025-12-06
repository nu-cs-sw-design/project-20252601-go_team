use actix_web::{test, web, App};
use orderbook::controller::controller::{get_open_orders, place_limit_order};
use rust_decimal::Decimal;
use serde_json::json;

#[actix_web::test]
async fn get_open_orders_returns_side_specific_orders() {
    let app = test::init_service(
        App::new()
            .route("/place_limit_order", web::post().to(place_limit_order))
            .route("/get_open_orders", web::get().to(get_open_orders)),
    )
    .await;

    // Seed opposite sides to ensure side filtering works.
    let buy_resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/place_limit_order")
            .set_json(json!({
                "symbol": "DEPTH",
                "side": "BUY",
                "price": "10.00",
                "quantity": "5",
                "tif": "GTC"
            }))
            .to_request(),
    )
    .await;
    assert!(buy_resp.status().is_success());

    let sell_resp = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/place_limit_order")
            .set_json(json!({
                "symbol": "DEPTH",
                "side": "SELL",
                "price": "11.00",
                "quantity": "3",
                "tif": "GTC"
            }))
            .to_request(),
    )
    .await;
    assert!(sell_resp.status().is_success());

    // Fetch BUY orders only.
    let buy_orders_req = test::TestRequest::get()
        .uri("/get_open_orders?symbol=DEPTH&side=BUY")
        .to_request();

    let buy_orders_resp = test::call_service(&app, buy_orders_req).await;
    assert!(buy_orders_resp.status().is_success());

    let buy_orders: Vec<orderbook::controller::controller::OrderDto> =
        test::read_body_json(buy_orders_resp).await;
    assert_eq!(buy_orders.len(), 1);
    assert_eq!(buy_orders[0].side, "BUY");
    assert_eq!(buy_orders[0].price, Some(Decimal::new(1000, 2)));
    assert_eq!(buy_orders[0].remaining_quantity, Decimal::new(5, 0));

    // Fetch SELL orders only.
    let sell_orders_req = test::TestRequest::get()
        .uri("/get_open_orders?symbol=DEPTH&side=SELL")
        .to_request();

    let sell_orders_resp = test::call_service(&app, sell_orders_req).await;
    assert!(sell_orders_resp.status().is_success());

    let sell_orders: Vec<orderbook::controller::controller::OrderDto> =
        test::read_body_json(sell_orders_resp).await;
    assert_eq!(sell_orders.len(), 1);
    assert_eq!(sell_orders[0].side, "SELL");
    assert_eq!(sell_orders[0].price, Some(Decimal::new(1100, 2)));
    assert_eq!(sell_orders[0].remaining_quantity, Decimal::new(3, 0));
}

#[actix_web::test]
async fn get_open_orders_validates_inputs() {
    let app =
        test::init_service(App::new().route("/get_open_orders", web::get().to(get_open_orders)))
            .await;

    let bad_side_req = test::TestRequest::get()
        .uri("/get_open_orders?symbol=DEPTH&side=HOLD")
        .to_request();

    let bad_side_resp = test::call_service(&app, bad_side_req).await;
    assert_eq!(
        bad_side_resp.status(),
        actix_web::http::StatusCode::BAD_REQUEST
    );

    let bad_symbol_req = test::TestRequest::get()
        .uri("/get_open_orders?symbol=&side=BUY")
        .to_request();

    let bad_symbol_resp = test::call_service(&app, bad_symbol_req).await;
    assert_eq!(
        bad_symbol_resp.status(),
        actix_web::http::StatusCode::BAD_REQUEST
    );
}

#[actix_web::test]
async fn get_open_orders_returns_404_for_missing_book() {
    let app =
        test::init_service(App::new().route("/get_open_orders", web::get().to(get_open_orders)))
            .await;

    let req = test::TestRequest::get()
        .uri("/get_open_orders?symbol=UNKNOWN&side=BUY")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
}
