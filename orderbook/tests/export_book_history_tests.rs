use actix_web::{test, web, App};
use orderbook::controller::controller::{export_book_history, place_limit_order, ExportResponse};
use serde_json::json;
use std::fs;
use uuid::Uuid;

#[actix_web::test]
async fn export_book_history_writes_csv_snapshot() {
    let app = test::init_service(
        App::new()
            .route("/place_limit_order", web::post().to(place_limit_order))
            .route("/export_book_history", web::post().to(export_book_history)),
    )
    .await;

    // Seed resting bids and asks so the snapshot has depth on both sides.
    let ask_req = test::TestRequest::post()
        .uri("/place_limit_order")
        .set_json(json!({
            "symbol": "EXPORT-BOOK",
            "side": "SELL",
            "price": "12.50",
            "quantity": "5",
            "tif": "GTC"
        }))
        .to_request();
    assert!(test::call_service(&app, ask_req)
        .await
        .status()
        .is_success());

    let bid_req = test::TestRequest::post()
        .uri("/place_limit_order")
        .set_json(json!({
            "symbol": "EXPORT-BOOK",
            "side": "BUY",
            "price": "10.00",
            "quantity": "3",
            "tif": "GTC"
        }))
        .to_request();
    assert!(test::call_service(&app, bid_req)
        .await
        .status()
        .is_success());

    let mut path = std::env::temp_dir();
    path.push(format!("export-book-history-{}.csv", Uuid::new_v4()));
    let path_str = path.to_string_lossy().to_string();

    let export_req = test::TestRequest::post()
        .uri("/export_book_history")
        .set_json(json!({
            "symbol": "EXPORT-BOOK",
            "path": path_str.clone()
        }))
        .to_request();

    let export_resp = test::call_service(&app, export_req).await;
    assert!(export_resp.status().is_success());

    let payload: ExportResponse = test::read_body_json(export_resp).await;
    assert!(payload.exported);
    assert_eq!(payload.path, path_str);
    assert_eq!(payload.count, 1);

    let contents = fs::read_to_string(&path).expect("exported snapshot to exist");
    let mut lines = contents.lines();
    assert_eq!(
        lines.next().unwrap_or_default(),
        "timestamp,symbol,bid_levels,ask_levels,recent_trades"
    );
    let data_line = lines.next().expect("snapshot row present");
    let columns: Vec<&str> = data_line.split(',').collect();
    assert_eq!(columns.len(), 5);
    assert_eq!(columns[1], "EXPORT-BOOK");

    fs::remove_file(&path).ok();
}

#[actix_web::test]
async fn export_book_history_returns_not_found_for_unknown_book() {
    let app = test::init_service(
        App::new().route("/export_book_history", web::post().to(export_book_history)),
    )
    .await;

    let mut path = std::env::temp_dir();
    path.push(format!(
        "export-book-history-missing-{}.csv",
        Uuid::new_v4()
    ));
    let path_clone = path.clone();
    let path_str = path.to_string_lossy().to_string();

    let export_req = test::TestRequest::post()
        .uri("/export_book_history")
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
async fn export_book_history_validates_inputs() {
    let app = test::init_service(
        App::new().route("/export_book_history", web::post().to(export_book_history)),
    )
    .await;

    let bad_symbol_req = test::TestRequest::post()
        .uri("/export_book_history")
        .set_json(json!({
            "symbol": " ",
            "path": "C:/tmp/book-history.csv"
        }))
        .to_request();
    let bad_symbol_resp = test::call_service(&app, bad_symbol_req).await;
    assert_eq!(
        bad_symbol_resp.status(),
        actix_web::http::StatusCode::BAD_REQUEST
    );

    let bad_path_req = test::TestRequest::post()
        .uri("/export_book_history")
        .set_json(json!({
            "symbol": "EXPORT-BOOK",
            "path": ""
        }))
        .to_request();
    let bad_path_resp = test::call_service(&app, bad_path_req).await;
    assert_eq!(
        bad_path_resp.status(),
        actix_web::http::StatusCode::BAD_REQUEST
    );
}

#[actix_web::test]
async fn export_book_history_creates_missing_directories() {
    let app = test::init_service(
        App::new()
            .route("/place_limit_order", web::post().to(place_limit_order))
            .route("/export_book_history", web::post().to(export_book_history)),
    )
    .await;

    // Seed a simple book state to snapshot.
    let ask_req = test::TestRequest::post()
        .uri("/place_limit_order")
        .set_json(json!({
            "symbol": "EXPORT-DIR-BOOK",
            "side": "SELL",
            "price": "33.00",
            "quantity": "1",
            "tif": "GTC"
        }))
        .to_request();
    assert!(test::call_service(&app, ask_req)
        .await
        .status()
        .is_success());

    let base_dir = std::env::temp_dir().join(format!("export-book-dir-{}", Uuid::new_v4()));
    let nested_dir = base_dir.join("history").join("snapshots");
    let file_path = nested_dir.join("book.csv");

    if base_dir.exists() {
        fs::remove_dir_all(&base_dir).ok();
    }

    assert!(!nested_dir.exists());

    let export_req = test::TestRequest::post()
        .uri("/export_book_history")
        .set_json(json!({
            "symbol": "EXPORT-DIR-BOOK",
            "path": file_path.to_string_lossy()
        }))
        .to_request();

    let export_resp = test::call_service(&app, export_req).await;
    assert!(export_resp.status().is_success());

    assert!(
        nested_dir.exists(),
        "expected export directories to be created"
    );
    assert!(file_path.exists(), "expected book history file to exist");

    fs::remove_dir_all(&base_dir).ok();
}
