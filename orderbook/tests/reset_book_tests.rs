use actix_web::{test, web, App};
use orderbook::controller::controller::{
    get_open_orders, place_limit_order, reset_book, OrderDto, PlaceLimitOrderResponse,
};
use serde_json::json;

#[actix_web::test]
async fn reset_book_clears_resting_orders() {
    let app = test::init_service(
        App::new()
            .route("/place_limit_order", web::post().to(place_limit_order))
            .route("/get_open_orders", web::get().to(get_open_orders))
            .route("/reset_book", web::post().to(reset_book)),
    )
    .await;

    let place_req = test::TestRequest::post()
        .uri("/place_limit_order")
        .set_json(json!({
            "symbol": "RESET-CLEAR",
            "side": "BUY",
            "price": "10.50",
            "quantity": "4",
            "tif": "GTC"
        }))
        .to_request();
    let place_resp = test::call_service(&app, place_req).await;
    assert!(place_resp.status().is_success());

    let placed: PlaceLimitOrderResponse = test::read_body_json(place_resp).await;
    assert!(placed.resting_order.is_some());

    let open_before_req = test::TestRequest::get()
        .uri("/get_open_orders?symbol=RESET-CLEAR&side=BUY")
        .to_request();
    let open_before_resp = test::call_service(&app, open_before_req).await;
    assert!(open_before_resp.status().is_success());
    let open_before: Vec<OrderDto> = test::read_body_json(open_before_resp).await;
    assert_eq!(open_before.len(), 1);

    let reset_req = test::TestRequest::post()
        .uri("/reset_book")
        .set_json(json!({
            "symbol": "RESET-CLEAR"
        }))
        .to_request();
    let reset_resp = test::call_service(&app, reset_req).await;
    assert!(reset_resp.status().is_success());
    let reset_done: bool = test::read_body_json(reset_resp).await;
    assert!(reset_done);

    let open_after_req = test::TestRequest::get()
        .uri("/get_open_orders?symbol=RESET-CLEAR&side=BUY")
        .to_request();
    let open_after_resp = test::call_service(&app, open_after_req).await;
    assert!(open_after_resp.status().is_success());
    let open_after: Vec<OrderDto> = test::read_body_json(open_after_resp).await;
    assert!(open_after.is_empty());
}

#[actix_web::test]
async fn reset_book_returns_false_when_missing() {
    let app = test::init_service(App::new().route("/reset_book", web::post().to(reset_book))).await;

    let reset_req = test::TestRequest::post()
        .uri("/reset_book")
        .set_json(json!({
            "symbol": "UNKNOWN-SYMBOL"
        }))
        .to_request();

    let reset_resp = test::call_service(&app, reset_req).await;
    assert!(reset_resp.status().is_success());
    let reset_done: bool = test::read_body_json(reset_resp).await;
    assert!(!reset_done);
}

#[actix_web::test]
async fn reset_book_rejects_empty_symbol() {
    let app = test::init_service(App::new().route("/reset_book", web::post().to(reset_book))).await;

    let reset_req = test::TestRequest::post()
        .uri("/reset_book")
        .set_json(json!({
            "symbol": ""
        }))
        .to_request();

    let reset_resp = test::call_service(&app, reset_req).await;
    assert_eq!(
        reset_resp.status(),
        actix_web::http::StatusCode::BAD_REQUEST
    );
}
