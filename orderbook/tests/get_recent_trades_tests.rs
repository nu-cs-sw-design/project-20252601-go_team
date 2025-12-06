use actix_web::{test, web, App};
use orderbook::controller::controller::{get_recent_trades, place_limit_order, place_market_order};
use rust_decimal::Decimal;
use serde_json::json;

#[actix_web::test]
async fn get_recent_trades_returns_latest_fills() {
    let app = test::init_service(
        App::new()
            .route("/place_limit_order", web::post().to(place_limit_order))
            .route("/place_market_order", web::post().to(place_market_order))
            .route("/get_recent_trades", web::get().to(get_recent_trades)),
    )
    .await;

    // Seed a resting sell order and then execute a market buy to produce a trade.
    let seed_req = test::TestRequest::post()
        .uri("/place_limit_order")
        .set_json(json!({
            "symbol": "TRADES-LATEST",
            "side": "SELL",
            "price": "15.00",
            "quantity": "5",
            "tif": "GTC"
        }))
        .to_request();
    let seed_resp = test::call_service(&app, seed_req).await;
    assert!(seed_resp.status().is_success());

    let market_req = test::TestRequest::post()
        .uri("/place_market_order")
        .set_json(json!({
            "symbol": "TRADES-LATEST",
            "side": "BUY",
            "quantity": "3",
            "tif": "IOC"
        }))
        .to_request();
    let market_resp = test::call_service(&app, market_req).await;
    assert!(market_resp.status().is_success());

    let trades_req = test::TestRequest::get()
        .uri("/get_recent_trades?symbol=TRADES-LATEST&limit=10")
        .to_request();
    let trades_resp = test::call_service(&app, trades_req).await;
    assert!(trades_resp.status().is_success());

    let trades: Vec<orderbook::controller::controller::TradeDto> =
        test::read_body_json(trades_resp).await;
    assert_eq!(trades.len(), 1);
    let trade = &trades[0];
    assert_eq!(trade.symbol, "TRADES-LATEST");
    assert_eq!(trade.quantity, Decimal::new(3, 0));
    assert_eq!(trade.price, Decimal::new(1500, 2));
}

#[actix_web::test]
async fn get_recent_trades_limits_results() {
    let app = test::init_service(
        App::new()
            .route("/place_limit_order", web::post().to(place_limit_order))
            .route("/place_market_order", web::post().to(place_market_order))
            .route("/get_recent_trades", web::get().to(get_recent_trades)),
    )
    .await;

    // Create resting liquidity that will be partially filled multiple times.
    let seed_req = test::TestRequest::post()
        .uri("/place_limit_order")
        .set_json(json!({
            "symbol": "TRADES-LIMIT",
            "side": "SELL",
            "price": "20.00",
            "quantity": "10",
            "tif": "GTC"
        }))
        .to_request();
    let seed_resp = test::call_service(&app, seed_req).await;
    assert!(seed_resp.status().is_success());

    for _ in 0..3 {
        let market_req = test::TestRequest::post()
            .uri("/place_market_order")
            .set_json(json!({
                "symbol": "TRADES-LIMIT",
                "side": "BUY",
                "quantity": "2",
                "tif": "IOC"
            }))
            .to_request();
        let resp = test::call_service(&app, market_req).await;
        assert!(resp.status().is_success());
    }

    let trades_req = test::TestRequest::get()
        .uri("/get_recent_trades?symbol=TRADES-LIMIT&limit=2")
        .to_request();
    let trades_resp = test::call_service(&app, trades_req).await;
    assert!(trades_resp.status().is_success());

    let trades: Vec<orderbook::controller::controller::TradeDto> =
        test::read_body_json(trades_resp).await;
    assert_eq!(trades.len(), 2);
}

#[actix_web::test]
async fn get_recent_trades_handles_bad_inputs() {
    let app = test::init_service(
        App::new().route("/get_recent_trades", web::get().to(get_recent_trades)),
    )
    .await;

    let bad_symbol_req = test::TestRequest::get()
        .uri("/get_recent_trades?symbol=&limit=5")
        .to_request();
    let bad_symbol_resp = test::call_service(&app, bad_symbol_req).await;
    assert_eq!(
        bad_symbol_resp.status(),
        actix_web::http::StatusCode::BAD_REQUEST
    );

    let missing_book_req = test::TestRequest::get()
        .uri("/get_recent_trades?symbol=UNKNOWN&limit=5")
        .to_request();
    let missing_book_resp = test::call_service(&app, missing_book_req).await;
    assert_eq!(
        missing_book_resp.status(),
        actix_web::http::StatusCode::NOT_FOUND
    );
}
