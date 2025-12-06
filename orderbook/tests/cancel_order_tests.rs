use actix_web::{test, web, App};
use orderbook::controller::controller::{cancel_order, place_limit_order, PlaceLimitOrderResponse};
use serde_json::json;

#[actix_web::test]
async fn cancel_existing_resting_order_returns_true() {
    let app = test::init_service(
        App::new()
            .route("/place_limit_order", web::post().to(place_limit_order))
            .route("/cancel_order", web::post().to(cancel_order)),
    )
    .await;

    let create_req = test::TestRequest::post()
        .uri("/place_limit_order")
        .set_json(json!({
            "symbol": "CANCEL-SUCCESS",
            "side": "SELL",
            "price": "250.00",
            "quantity": "4",
            "tif": "GTC"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert!(create_resp.status().is_success());

    let order_payload: PlaceLimitOrderResponse = test::read_body_json(create_resp).await;
    let order_id = order_payload.order_id.clone();

    let cancel_req = test::TestRequest::post()
        .uri("/cancel_order")
        .set_json(json!({
            "symbol": "CANCEL-SUCCESS",
            "order_id": order_id
        }))
        .to_request();

    let cancel_resp = test::call_service(&app, cancel_req).await;
    assert!(cancel_resp.status().is_success());
    let cancelled: bool = test::read_body_json(cancel_resp).await;
    assert!(cancelled);

    // Cancelling again should return false now that it's gone
    let second_cancel = test::TestRequest::post()
        .uri("/cancel_order")
        .set_json(json!({
            "symbol": "CANCEL-SUCCESS",
            "order_id": order_id.clone()
        }))
        .to_request();

    let second_resp = test::call_service(&app, second_cancel).await;
    assert!(second_resp.status().is_success());
    let second_result: bool = test::read_body_json(second_resp).await;
    assert!(!second_result);
}

#[actix_web::test]
async fn cancel_on_missing_symbol_returns_false() {
    let app =
        test::init_service(App::new().route("/cancel_order", web::post().to(cancel_order))).await;

    let cancel_req = test::TestRequest::post()
        .uri("/cancel_order")
        .set_json(json!({
            "symbol": "NO-SUCH-SYMBOL",
            "order_id": "ord-missing"
        }))
        .to_request();

    let cancel_resp = test::call_service(&app, cancel_req).await;
    assert!(cancel_resp.status().is_success());

    let cancelled: bool = test::read_body_json(cancel_resp).await;
    assert!(!cancelled);
}

#[actix_web::test]
async fn cancel_with_wrong_order_id_returns_false() {
    let app = test::init_service(
        App::new()
            .route("/place_limit_order", web::post().to(place_limit_order))
            .route("/cancel_order", web::post().to(cancel_order)),
    )
    .await;

    let create_req = test::TestRequest::post()
        .uri("/place_limit_order")
        .set_json(json!({
            "symbol": "CANCEL-WRONG-ID",
            "side": "BUY",
            "price": "42.42",
            "quantity": "2",
            "tif": "GTC"
        }))
        .to_request();

    let create_resp = test::call_service(&app, create_req).await;
    assert!(create_resp.status().is_success());

    let payload: PlaceLimitOrderResponse = test::read_body_json(create_resp).await;
    assert!(payload.resting_order.is_some());

    let cancel_req = test::TestRequest::post()
        .uri("/cancel_order")
        .set_json(json!({
            "symbol": "CANCEL-WRONG-ID",
            "order_id": "ord-wrong-id"
        }))
        .to_request();

    let cancel_resp = test::call_service(&app, cancel_req).await;
    assert!(cancel_resp.status().is_success());

    let cancelled: bool = test::read_body_json(cancel_resp).await;
    assert!(!cancelled);
}
