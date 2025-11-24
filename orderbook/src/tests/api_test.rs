use actix_web::{test, App, web};
use crate::place_limit_order;

#[actix_web::test]
async fn test_place_limit_order_success() {
    // 1. Initialize the service with the specific route we want to test
    let app = test::init_service(
        App::new().route("/place_limit_order", web::post().to(place_limit_order))
    ).await;

    // 2. Create a POST request with valid JSON payload
    let req = test::TestRequest::post()
        .uri("/place_limit_order")
        .set_json(serde_json::json!({
            "symbol": "AAPL",
            "side": "BUY",
            "price": "150.50",
            "quantity": "10",
            "tif": "GTC"
        }))
        .to_request();

    // 3. Call the service
    let resp = test::call_service(&app, req).await;

    // 4. Assertions
    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body).unwrap();
    
    // Check if the response body looks like an order ID (e.g., "ord-AAPL-...")
    assert!(body_str.starts_with("ord-AAPL-"));
}

#[actix_web::test]
async fn test_place_limit_order_invalid_side() {
    let app = test::init_service(
        App::new().route("/place_limit_order", web::post().to(place_limit_order))
    ).await;

    let req = test::TestRequest::post()
        .uri("/place_limit_order")
        .set_json(serde_json::json!({
            "symbol": "AAPL",
            "side": "INVALID",
            "price": "150.50",
            "quantity": "10",
            "tif": "GTC"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    
    // Should return 400 Bad Request based on our implementation
    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
}
