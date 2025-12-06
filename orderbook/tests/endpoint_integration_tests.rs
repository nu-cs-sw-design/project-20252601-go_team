use actix_web::{test, web, App};
use orderbook::controller::controller::{
    cancel_order, get_open_orders, get_order, modify_order, place_limit_order, place_market_order,
    OrderDto, PlaceLimitOrderResponse, PlaceMarketOrderResponse,
};
use rust_decimal::Decimal;
use serde_json::json;

#[actix_web::test]
async fn limit_order_round_trip_get_order() {
    let app = test::init_service(
        App::new()
            .route("/place_limit_order", web::post().to(place_limit_order))
            .route("/get_order", web::get().to(get_order)),
    )
    .await;

    let place_req = test::TestRequest::post()
        .uri("/place_limit_order")
        .set_json(json!({
            "symbol": "FLOW-ROUND",
            "side": "BUY",
            "price": "123.45",
            "quantity": "8",
            "tif": "GTC"
        }))
        .to_request();

    let place_resp = test::call_service(&app, place_req).await;
    assert!(place_resp.status().is_success());

    let placed: PlaceLimitOrderResponse = test::read_body_json(place_resp).await;
    let order_id = placed.order_id.clone();

    let get_req = test::TestRequest::get()
        .uri(&format!(
            "/get_order?symbol=FLOW-ROUND&order_id={}",
            order_id
        ))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert!(get_resp.status().is_success());

    let fetched: OrderDto = test::read_body_json(get_resp).await;
    assert_eq!(fetched.symbol, "FLOW-ROUND");
    assert_eq!(fetched.side, "BUY");
    assert_eq!(fetched.price, Some(Decimal::new(12345, 2)));
    assert_eq!(fetched.remaining_quantity, Decimal::new(8, 0));
    assert_eq!(fetched.tif, "GTC");
}

#[actix_web::test]
async fn open_orders_track_multiple_resting_orders() {
    let app = test::init_service(
        App::new()
            .route("/place_limit_order", web::post().to(place_limit_order))
            .route("/get_open_orders", web::get().to(get_open_orders)),
    )
    .await;

    for (price, qty) in [("100.00", "5"), ("99.75", "3")] {
        let req = test::TestRequest::post()
            .uri("/place_limit_order")
            .set_json(json!({
                "symbol": "FLOW-DEPTH",
                "side": "BUY",
                "price": price,
                "quantity": qty,
                "tif": "GTC"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    let open_req = test::TestRequest::get()
        .uri("/get_open_orders?symbol=FLOW-DEPTH&side=BUY")
        .to_request();

    let open_resp = test::call_service(&app, open_req).await;
    assert!(open_resp.status().is_success());

    let orders: Vec<OrderDto> = test::read_body_json(open_resp).await;
    assert_eq!(orders.len(), 2);
    assert!(orders.iter().all(|order| order.side == "BUY"));

    let mut prices: Vec<Decimal> = orders
        .iter()
        .map(|order| order.price.expect("limit orders should have a price"))
        .collect();
    prices.sort();
    assert_eq!(prices, vec![Decimal::new(9975, 2), Decimal::new(10000, 2)]);
}

#[actix_web::test]
async fn market_order_consumes_best_prices_first() {
    let app = test::init_service(
        App::new()
            .route("/place_limit_order", web::post().to(place_limit_order))
            .route("/place_market_order", web::post().to(place_market_order))
            .route("/get_open_orders", web::get().to(get_open_orders)),
    )
    .await;

    // Seed sell-side liquidity at two price levels.
    for (price, qty) in [("100.00", "5"), ("101.00", "4")] {
        let req = test::TestRequest::post()
            .uri("/place_limit_order")
            .set_json(json!({
                "symbol": "FLOW-MKT",
                "side": "SELL",
                "price": price,
                "quantity": qty,
                "tif": "GTC"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    let market_req = test::TestRequest::post()
        .uri("/place_market_order")
        .set_json(json!({
            "symbol": "FLOW-MKT",
            "side": "BUY",
            "quantity": "7",
            "tif": "IOC"
        }))
        .to_request();

    let market_resp = test::call_service(&app, market_req).await;
    assert!(market_resp.status().is_success());

    let executed: PlaceMarketOrderResponse = test::read_body_json(market_resp).await;
    assert_eq!(executed.executed_quantity, Decimal::new(7, 0));
    assert!(executed.fully_filled);
    assert_eq!(executed.tif, "IOC");
    assert!(executed.trades.len() >= 2);
    assert!(executed
        .trades
        .iter()
        .any(|trade| trade.price == Decimal::new(10000, 2)));
    assert!(executed
        .trades
        .iter()
        .any(|trade| trade.price == Decimal::new(10100, 2)));

    let remaining_req = test::TestRequest::get()
        .uri("/get_open_orders?symbol=FLOW-MKT&side=SELL")
        .to_request();
    let remaining_resp = test::call_service(&app, remaining_req).await;
    assert!(remaining_resp.status().is_success());

    let remaining: Vec<OrderDto> = test::read_body_json(remaining_resp).await;
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].price, Some(Decimal::new(10100, 2)));
    assert_eq!(remaining[0].remaining_quantity, Decimal::new(2, 0));
}

#[actix_web::test]
async fn modify_order_updates_order_details() {
    let app = test::init_service(
        App::new()
            .route("/place_limit_order", web::post().to(place_limit_order))
            .route("/modify_order", web::post().to(modify_order))
            .route("/get_order", web::get().to(get_order)),
    )
    .await;

    let place_req = test::TestRequest::post()
        .uri("/place_limit_order")
        .set_json(json!({
            "symbol": "FLOW-MOD",
            "side": "SELL",
            "price": "55.50",
            "quantity": "6",
            "tif": "GTC"
        }))
        .to_request();
    let place_resp = test::call_service(&app, place_req).await;
    assert!(place_resp.status().is_success());

    let placed: PlaceLimitOrderResponse = test::read_body_json(place_resp).await;
    let order_id = placed.order_id.clone();

    let modify_req = test::TestRequest::post()
        .uri("/modify_order")
        .set_json(json!({
            "symbol": "FLOW-MOD",
            "order_id": order_id.clone(),
            "new_price": "60.25",
            "new_quantity": "4"
        }))
        .to_request();

    let modify_resp = test::call_service(&app, modify_req).await;
    assert!(modify_resp.status().is_success());
    let modified: bool = test::read_body_json(modify_resp).await;
    assert!(modified);

    let get_req = test::TestRequest::get()
        .uri(&format!("/get_order?symbol=FLOW-MOD&order_id={}", order_id))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert!(get_resp.status().is_success());
    let fetched: OrderDto = test::read_body_json(get_resp).await;
    assert_eq!(fetched.price, Some(Decimal::new(6025, 2)));
    assert_eq!(fetched.remaining_quantity, Decimal::new(4, 0));
}

#[actix_web::test]
async fn cancel_order_clears_resting_liquidity() {
    let app = test::init_service(
        App::new()
            .route("/place_limit_order", web::post().to(place_limit_order))
            .route("/cancel_order", web::post().to(cancel_order))
            .route("/get_open_orders", web::get().to(get_open_orders)),
    )
    .await;

    let place_req = test::TestRequest::post()
        .uri("/place_limit_order")
        .set_json(json!({
            "symbol": "FLOW-CANCEL",
            "side": "SELL",
            "price": "75.00",
            "quantity": "2",
            "tif": "GTC"
        }))
        .to_request();

    let place_resp = test::call_service(&app, place_req).await;
    assert!(place_resp.status().is_success());
    let placed: PlaceLimitOrderResponse = test::read_body_json(place_resp).await;

    let cancel_req = test::TestRequest::post()
        .uri("/cancel_order")
        .set_json(json!({
            "symbol": "FLOW-CANCEL",
            "order_id": placed.order_id
        }))
        .to_request();

    let cancel_resp = test::call_service(&app, cancel_req).await;
    assert!(cancel_resp.status().is_success());
    let cancelled: bool = test::read_body_json(cancel_resp).await;
    assert!(cancelled);

    let open_req = test::TestRequest::get()
        .uri("/get_open_orders?symbol=FLOW-CANCEL&side=SELL")
        .to_request();
    let open_resp = test::call_service(&app, open_req).await;
    assert!(open_resp.status().is_success());

    let orders: Vec<OrderDto> = test::read_body_json(open_resp).await;
    assert!(orders.is_empty());
}
