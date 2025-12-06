use actix_web::{http::StatusCode, test, web, App};
use orderbook::controller::controller::{
    cancel_order, export_book_history, export_trades, get_open_orders, get_order,
    get_recent_trades, health, modify_order, place_limit_order, place_market_order, reset_book,
    ExportResponse, OrderDto, PlaceLimitOrderResponse, PlaceMarketOrderResponse, TradeDto,
};
use rust_decimal::Decimal;
use serde_json::json;
use std::fs;
use uuid::Uuid;

#[actix_web::test]
async fn end_to_end_endpoint_flow() {
    let app = test::init_service(
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
            .route("/export_book_history", web::post().to(export_book_history)),
    )
    .await;

    // Health check
    let health_req = test::TestRequest::post()
        .uri("/health")
        .set_payload("ping")
        .to_request();
    let health_resp = test::call_service(&app, health_req).await;
    assert_eq!(health_resp.status(), StatusCode::OK);
    let health_body = test::read_body(health_resp).await;
    assert_eq!(health_body, "Endpoint is healthy");

    // Seed two resting sell orders
    let first_limit_req = test::TestRequest::post()
        .uri("/place_limit_order")
        .set_json(json!({
            "symbol": "ENDPOINT-FLOW",
            "side": "SELL",
            "price": "100.00",
            "quantity": "5",
            "tif": "GTC"
        }))
        .to_request();
    let first_limit_resp = test::call_service(&app, first_limit_req).await;
    assert!(first_limit_resp.status().is_success());
    let first_limit: PlaceLimitOrderResponse = test::read_body_json(first_limit_resp).await;
    assert!(first_limit.resting_order.is_some());
    let first_order_id = first_limit.order_id.clone();

    let second_limit_req = test::TestRequest::post()
        .uri("/place_limit_order")
        .set_json(json!({
            "symbol": "ENDPOINT-FLOW",
            "side": "SELL",
            "price": "101.00",
            "quantity": "2",
            "tif": "GTC"
        }))
        .to_request();
    let second_limit_resp = test::call_service(&app, second_limit_req).await;
    assert!(second_limit_resp.status().is_success());
    let second_limit: PlaceLimitOrderResponse = test::read_body_json(second_limit_resp).await;
    let second_order_id = second_limit.order_id.clone();

    // Modify the second order
    let modify_req = test::TestRequest::post()
        .uri("/modify_order")
        .set_json(json!({
            "symbol": "ENDPOINT-FLOW",
            "order_id": second_order_id.clone(),
            "new_price": "99.00",
            "new_quantity": "4"
        }))
        .to_request();
    let modify_resp = test::call_service(&app, modify_req).await;
    assert!(modify_resp.status().is_success());
    let modified: bool = test::read_body_json(modify_resp).await;
    assert!(modified);

    // Confirm order details after modification
    let get_modified_req = test::TestRequest::get()
        .uri(&format!(
            "/get_order?symbol=ENDPOINT-FLOW&order_id={}",
            second_order_id
        ))
        .to_request();
    let get_modified_resp = test::call_service(&app, get_modified_req).await;
    assert!(get_modified_resp.status().is_success());
    let modified_order: OrderDto = test::read_body_json(get_modified_resp).await;
    assert_eq!(modified_order.price, Some(Decimal::new(9900, 2)));
    assert_eq!(modified_order.remaining_quantity, Decimal::new(4, 0));

    // Fetch open orders to ensure both are present
    let open_req = test::TestRequest::get()
        .uri("/get_open_orders?symbol=ENDPOINT-FLOW&side=SELL")
        .to_request();
    let open_resp = test::call_service(&app, open_req).await;
    assert!(open_resp.status().is_success());
    let open_orders: Vec<OrderDto> = test::read_body_json(open_resp).await;
    assert_eq!(open_orders.len(), 2);

    // Execute a market buy that partially fills the modified order
    let market_req = test::TestRequest::post()
        .uri("/place_market_order")
        .set_json(json!({
            "symbol": "ENDPOINT-FLOW",
            "side": "BUY",
            "quantity": "3",
            "tif": "IOC"
        }))
        .to_request();
    let market_resp = test::call_service(&app, market_req).await;
    assert!(market_resp.status().is_success());
    let market_result: PlaceMarketOrderResponse = test::read_body_json(market_resp).await;
    assert_eq!(market_result.executed_quantity, Decimal::new(3, 0));
    assert!(market_result.fully_filled);
    assert!(!market_result.trades.is_empty());

    // Updated remaining quantity on modified order should reflect partial fill
    let get_after_trade_req = test::TestRequest::get()
        .uri(&format!(
            "/get_order?symbol=ENDPOINT-FLOW&order_id={}",
            second_order_id
        ))
        .to_request();
    let get_after_trade_resp = test::call_service(&app, get_after_trade_req).await;
    assert!(get_after_trade_resp.status().is_success());
    let post_trade_order: OrderDto = test::read_body_json(get_after_trade_resp).await;
    assert_eq!(post_trade_order.remaining_quantity, Decimal::new(1, 0));

    // Recent trades endpoint should report the executed trade(s)
    let recent_req = test::TestRequest::get()
        .uri("/get_recent_trades?symbol=ENDPOINT-FLOW&limit=10")
        .to_request();
    let recent_resp = test::call_service(&app, recent_req).await;
    assert!(recent_resp.status().is_success());
    let recent_trades: Vec<TradeDto> = test::read_body_json(recent_resp).await;
    assert!(!recent_trades.is_empty());
    assert!(recent_trades
        .iter()
        .all(|trade| trade.symbol == "ENDPOINT-FLOW"));

    // Export trades to a temp file
    let mut trades_path = std::env::temp_dir();
    trades_path.push(format!("full-flow-trades-{}.csv", Uuid::new_v4()));
    let trades_path_str = trades_path.to_string_lossy().to_string();
    let export_trades_req = test::TestRequest::post()
        .uri("/export_trades")
        .set_json(json!({
            "symbol": "ENDPOINT-FLOW",
            "path": trades_path_str.clone()
        }))
        .to_request();
    let export_trades_resp = test::call_service(&app, export_trades_req).await;
    assert!(export_trades_resp.status().is_success());
    let trades_payload: ExportResponse = test::read_body_json(export_trades_resp).await;
    assert!(trades_payload.exported);
    assert!(trades_payload.count >= 1);
    let trades_csv = fs::read_to_string(&trades_path).expect("trades export created");
    assert!(trades_csv.contains("trade_id,symbol"));

    // Export book history to a temp file
    let mut history_path = std::env::temp_dir();
    history_path.push(format!("full-flow-history-{}.csv", Uuid::new_v4()));
    let history_path_str = history_path.to_string_lossy().to_string();
    let export_history_req = test::TestRequest::post()
        .uri("/export_book_history")
        .set_json(json!({
            "symbol": "ENDPOINT-FLOW",
            "path": history_path_str.clone()
        }))
        .to_request();
    let export_history_resp = test::call_service(&app, export_history_req).await;
    assert!(export_history_resp.status().is_success());
    let history_payload: ExportResponse = test::read_body_json(export_history_resp).await;
    assert!(history_payload.exported);
    assert_eq!(history_payload.count, 1);
    let history_csv = fs::read_to_string(&history_path).expect("history export created");
    assert!(history_csv.contains("timestamp,symbol,bid_levels,ask_levels,recent_trades"));

    // Cancel the untouched resting order
    let cancel_req = test::TestRequest::post()
        .uri("/cancel_order")
        .set_json(json!({
            "symbol": "ENDPOINT-FLOW",
            "order_id": first_order_id
        }))
        .to_request();
    let cancel_resp = test::call_service(&app, cancel_req).await;
    assert!(cancel_resp.status().is_success());
    let cancelled: bool = test::read_body_json(cancel_resp).await;
    assert!(cancelled);

    // Reset the book to clear any remaining state
    let reset_req = test::TestRequest::post()
        .uri("/reset_book")
        .set_json(json!({
            "symbol": "ENDPOINT-FLOW"
        }))
        .to_request();
    let reset_resp = test::call_service(&app, reset_req).await;
    assert!(reset_resp.status().is_success());
    let reset_done: bool = test::read_body_json(reset_resp).await;
    assert!(reset_done);

    let open_after_reset_req = test::TestRequest::get()
        .uri("/get_open_orders?symbol=ENDPOINT-FLOW&side=SELL")
        .to_request();
    let open_after_reset_resp = test::call_service(&app, open_after_reset_req).await;
    assert!(open_after_reset_resp.status().is_success());
    let open_after_reset: Vec<OrderDto> = test::read_body_json(open_after_reset_resp).await;
    assert!(open_after_reset.is_empty());

    // Cleanup temp files
    fs::remove_file(trades_path).ok();
    fs::remove_file(history_path).ok();
}
