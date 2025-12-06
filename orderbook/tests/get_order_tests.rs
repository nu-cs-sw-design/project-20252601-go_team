use actix_web::{test, web, App};
use orderbook::controller::controller::{get_order, place_limit_order, PlaceLimitOrderResponse};
use rust_decimal::Decimal;
use serde_json::json;

#[actix_web::test]
async fn get_order_returns_resting_limit() {
    let app = test::init_service(
        App::new()
            .route("/place_limit_order", web::post().to(place_limit_order))
            .route("/get_order", web::get().to(get_order)),
    )
    .await;

    let create_req = test::TestRequest::post()
        .uri("/place_limit_order")
        .set_json(json!({
            "symbol": "GET-OK",
            "side": "BUY",
            "price": "99.99",
            "quantity": "7",
            "tif": "GTC"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert!(create_resp.status().is_success());

    let payload: PlaceLimitOrderResponse = test::read_body_json(create_resp).await;
    let order_id = payload.order_id.clone();

    let get_req = test::TestRequest::get()
        .uri(&format!(
            "/get_order?symbol={symbol}&order_id={order_id}",
            symbol = "GET-OK",
            order_id = order_id
        ))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert!(get_resp.status().is_success());

    let order_dto: orderbook::controller::controller::OrderDto =
        test::read_body_json(get_resp).await;
    assert_eq!(order_dto.symbol, "GET-OK");
    assert_eq!(order_dto.order_id, payload.order_id);
    assert_eq!(order_dto.price, Some(Decimal::new(9999, 2)));
    assert_eq!(order_dto.remaining_quantity, Decimal::new(7, 0));
    assert_eq!(order_dto.side, "BUY");
}

#[actix_web::test]
async fn get_order_returns_404_when_missing() {
    let app = test::init_service(App::new().route("/get_order", web::get().to(get_order))).await;

    let req = test::TestRequest::get()
        .uri("/get_order?symbol=UNKNOWN&order_id=ord-missing")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn get_order_rejects_bad_inputs() {
    let app = test::init_service(App::new().route("/get_order", web::get().to(get_order))).await;

    let req = test::TestRequest::get()
        .uri("/get_order?symbol=&order_id=")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
}
