use actix_web::{test, web, App};
use orderbook::controller::controller::{
    export_trades, place_limit_order, place_market_order, ExportResponse,
};
use serde_json::json;
use std::fs;
use uuid::Uuid;

#[actix_web::test]
async fn export_trades_writes_csv_with_trades() {
    let app = test::init_service(
        App::new()
            .route("/place_limit_order", web::post().to(place_limit_order))
            .route("/place_market_order", web::post().to(place_market_order))
            .route("/export_trades", web::post().to(export_trades)),
    )
    .await;

    // Seed resting liquidity.
    let seed_req = test::TestRequest::post()
        .uri("/place_limit_order")
        .set_json(json!({
            "symbol": "EXPORT-CSV",
            "side": "SELL",
            "price": "12.50",
            "quantity": "5",
            "tif": "GTC"
        }))
        .to_request();
    let seed_resp = test::call_service(&app, seed_req).await;
    assert!(seed_resp.status().is_success());

    // Execute a market order to generate trade history.
    let taker_req = test::TestRequest::post()
        .uri("/place_market_order")
        .set_json(json!({
            "symbol": "EXPORT-CSV",
            "side": "BUY",
            "quantity": "3",
            "tif": "IOC"
        }))
        .to_request();
    let taker_resp = test::call_service(&app, taker_req).await;
    assert!(taker_resp.status().is_success());

    let mut path = std::env::temp_dir();
    path.push(format!("export-trades-{}.csv", Uuid::new_v4()));
    let path_str = path.to_string_lossy().to_string();

    let export_req = test::TestRequest::post()
        .uri("/export_trades")
        .set_json(json!({
            "symbol": "EXPORT-CSV",
            "path": path_str.clone()
        }))
        .to_request();

    let export_resp = test::call_service(&app, export_req).await;
    assert!(export_resp.status().is_success());

    let payload: ExportResponse = test::read_body_json(export_resp).await;
    assert!(payload.exported);
    assert_eq!(payload.path, path_str);
    assert!(payload.count >= 1);

    let contents = fs::read_to_string(&path).expect("export file to exist");
    assert!(contents.contains("trade_id,symbol"));
    assert!(contents.contains("EXPORT-CSV"));
    assert!(contents.contains("3"));

    fs::remove_file(&path).ok();
}

#[actix_web::test]
async fn export_trades_creates_missing_directories() {
    let app = test::init_service(
        App::new()
            .route("/place_limit_order", web::post().to(place_limit_order))
            .route("/place_market_order", web::post().to(place_market_order))
            .route("/export_trades", web::post().to(export_trades)),
    )
    .await;

    // Seed liquidity and generate a trade to ensure the export has content.
    let seed_req = test::TestRequest::post()
        .uri("/place_limit_order")
        .set_json(json!({
            "symbol": "EXPORT-DIR",
            "side": "SELL",
            "price": "20.00",
            "quantity": "4",
            "tif": "GTC"
        }))
        .to_request();
    assert!(test::call_service(&app, seed_req)
        .await
        .status()
        .is_success());

    let taker_req = test::TestRequest::post()
        .uri("/place_market_order")
        .set_json(json!({
            "symbol": "EXPORT-DIR",
            "side": "BUY",
            "quantity": "2",
            "tif": "IOC"
        }))
        .to_request();
    assert!(test::call_service(&app, taker_req)
        .await
        .status()
        .is_success());

    let base_dir = std::env::temp_dir().join(format!("export-trades-dir-{}", Uuid::new_v4()));
    let nested_dir = base_dir.join("nested").join("exports");
    let file_path = nested_dir.join("trades.csv");

    if base_dir.exists() {
        fs::remove_dir_all(&base_dir).ok();
    }

    assert!(!nested_dir.exists());

    let export_req = test::TestRequest::post()
        .uri("/export_trades")
        .set_json(json!({
            "symbol": "EXPORT-DIR",
            "path": file_path.to_string_lossy()
        }))
        .to_request();

    let export_resp = test::call_service(&app, export_req).await;
    assert!(export_resp.status().is_success());

    assert!(
        nested_dir.exists(),
        "expected intermediate directories to be created"
    );
    assert!(file_path.exists(), "expected export file to exist");

    fs::remove_dir_all(&base_dir).ok();
}

#[actix_web::test]
async fn export_trades_returns_not_found_for_unknown_book() {
    let app =
        test::init_service(App::new().route("/export_trades", web::post().to(export_trades))).await;

    let mut path = std::env::temp_dir();
    path.push(format!("export-trades-missing-{}.csv", Uuid::new_v4()));
    let path_clone = path.clone();
    let path_str = path.to_string_lossy().to_string();

    let export_req = test::TestRequest::post()
        .uri("/export_trades")
        .set_json(json!({
            "symbol": "UNKNOWN-SYMBOL",
            "path": path_str.clone()
        }))
        .to_request();

    let export_resp = test::call_service(&app, export_req).await;
    assert_eq!(export_resp.status(), actix_web::http::StatusCode::NOT_FOUND);

    assert!(!path_clone.exists());
}

#[actix_web::test]
async fn export_trades_validates_inputs() {
    let app =
        test::init_service(App::new().route("/export_trades", web::post().to(export_trades))).await;

    let bad_symbol_req = test::TestRequest::post()
        .uri("/export_trades")
        .set_json(json!({
            "symbol": " ",
            "path": "C:/tmp/should-not-write.csv"
        }))
        .to_request();
    let bad_symbol_resp = test::call_service(&app, bad_symbol_req).await;
    assert_eq!(
        bad_symbol_resp.status(),
        actix_web::http::StatusCode::BAD_REQUEST
    );

    let bad_path_req = test::TestRequest::post()
        .uri("/export_trades")
        .set_json(json!({
            "symbol": "EXPORT-CSV",
            "path": ""
        }))
        .to_request();
    let bad_path_resp = test::call_service(&app, bad_path_req).await;
    assert_eq!(
        bad_path_resp.status(),
        actix_web::http::StatusCode::BAD_REQUEST
    );
}
