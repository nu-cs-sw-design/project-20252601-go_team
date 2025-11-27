use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::model::{Side, TimeInForce, Stock, LimitOrder, MarketOrder};

// --- DTOs for Responses ---

#[derive(Serialize)]
pub struct TradeDto {
    pub trade_id: String,
    pub symbol: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub taker_order_id: String,
    pub maker_order_id: String,
    pub timestamp: i64,
}

#[derive(Serialize)]
pub struct OrderDto {
    pub order_id: String,
    pub symbol: String,
    pub side: String,
    pub quantity: Decimal,
    pub remaining_quantity: Decimal,
    pub price: Option<Decimal>, // None for MarketOrder
    pub tif: String,
    pub timestamp: i64,
    pub type_: String, // "LIMIT" or "MARKET"
}

// --- Request Structs ---

#[derive(Deserialize)]
pub struct PlaceLimitOrderRequest {
    pub symbol: String,
    pub side: String, // Expecting "BUY" or "SELL"
    pub price: Decimal,
    pub quantity: Decimal,
    pub tif: String, // Expecting "GTC", "IOC", "FOK"
}

#[derive(Deserialize)]
pub struct PlaceMarketOrderRequest {
    pub symbol: String,
    pub side: String,
    pub quantity: Decimal,
    pub tif: String,
}

#[derive(Deserialize)]
pub struct CancelOrderRequest {
    pub symbol: String,
    pub order_id: String,
}

#[derive(Deserialize)]
pub struct ModifyOrderRequest {
    pub symbol: String,
    pub order_id: String,
    pub new_price: Decimal,
    pub new_quantity: Decimal,
}

#[derive(Deserialize)]
pub struct GetOrderRequest {
    pub symbol: String,
    pub order_id: String,
}

#[derive(Deserialize)]
pub struct GetOpenOrdersRequest {
    pub symbol: String,
    pub side: String,
}

#[derive(Deserialize)]
pub struct ResetBookRequest {
    pub symbol: String,
}

#[derive(Deserialize)]
pub struct GetRecentTradesRequest {
    pub symbol: String,
    pub limit: i32,
}

#[derive(Deserialize)]
pub struct ExportRequest {
    pub symbol: String,
    pub path: String,
}

// --- Handlers ---

pub async fn health(body: String) -> impl Responder {
    println!("Received content: {}", body);
    HttpResponse::Ok().body("Endpoint is healthy")
}

pub async fn place_limit_order(req: web::Json<PlaceLimitOrderRequest>) -> impl Responder {
    println!("--- Placing Limit Order ---");

    // 1. Parse Side Enum
    let side: Side = match req.side.to_uppercase().as_str() {
        "BUY" => Side::BUY,
        "SELL" => Side::SELL,
        _ => return HttpResponse::BadRequest().body("Invalid side. Must be BUY or SELL"),
    };

    // 2. Parse TimeInForce Enum
    let tif: TimeInForce = match req.tif.to_uppercase().as_str() {
        "GTC" => TimeInForce::GTC,
        "IOC" => TimeInForce::IOC,
        "FOK" => TimeInForce::FOK,
        _ => return HttpResponse::BadRequest().body("Invalid TIF. Must be GTC, IOC, or FOK"),
    };

    // 3. Create Asset (Mocking lookup since we don't have the simulator state here)
    let asset: Arc<Stock> = Arc::new(Stock::new(String::from(&req.symbol), String::from("Unknown Name"), String::from("Created via API")));

    // 4. Generate Metadata
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let order_id = format!("ord-{}-{}", req.symbol, timestamp);

    // 5. Instantiate the actual Domain Object
    let order = LimitOrder::new(
        order_id.clone(),
        asset,
        side,
        req.quantity,
        timestamp,
        tif,
        req.price,
    );

    println!("Successfully created internal order object:");
    println!("{:#?}", order);
    
    // TODO: Add order to OrderBook via Model/Simulator

    HttpResponse::Ok().body(order_id)
}

pub async fn place_market_order(req: web::Json<PlaceMarketOrderRequest>) -> impl Responder {
    println!("--- Placing Market Order ---");
    println!("Symbol: {}, Side: {}, Qty: {}, TIF: {}", req.symbol, req.side, req.quantity, req.tif);

    // Mocking logic
    // TODO: Call model to place market order and get trades

    let trades = vec![
        TradeDto {
            trade_id: "trade-1".to_string(),
            symbol: req.symbol.clone(),
            price: Decimal::new(100, 0),
            quantity: req.quantity,
            taker_order_id: "taker-1".to_string(),
            maker_order_id: "maker-1".to_string(),
            timestamp: 1234567890,
        }
    ];

    println!("Market Order placed. Trades generated: {}", trades.len());
    HttpResponse::Ok().json(trades)
}

pub async fn cancel_order(req: web::Json<CancelOrderRequest>) -> impl Responder {
    println!("--- Cancelling Order ---");
    println!("Symbol: {}, OrderID: {}", req.symbol, req.order_id);

    // TODO: Call model to cancel order
    let success = true;

    println!("Order cancelled status: {}", success);
    HttpResponse::Ok().json(success)
}

pub async fn modify_order(req: web::Json<ModifyOrderRequest>) -> impl Responder {
    println!("--- Modifying Order ---");
    println!("Symbol: {}, OrderID: {}, NewPrice: {}, NewQty: {}", req.symbol, req.order_id, req.new_price, req.new_quantity);

    // TODO: Call model to modify order
    let success = true;

    println!("Order modified status: {}", success);
    HttpResponse::Ok().json(success)
}

pub async fn get_order(info: web::Query<GetOrderRequest>) -> impl Responder {
    println!("--- Get Order ---");
    println!("Symbol: {}, OrderID: {}", info.symbol, info.order_id);

    // TODO: Fetch order from model
    
    let order_dto = OrderDto {
        order_id: info.order_id.clone(),
        symbol: info.symbol.clone(),
        side: "BUY".to_string(),
        quantity: Decimal::new(10, 0),
        remaining_quantity: Decimal::new(5, 0),
        price: Some(Decimal::new(150, 0)),
        tif: "GTC".to_string(),
        timestamp: 1234567890,
        type_: "LIMIT".to_string(),
    };

    println!("Order found: {:?}", order_dto.order_id);
    HttpResponse::Ok().json(order_dto)
}

pub async fn get_open_orders(info: web::Query<GetOpenOrdersRequest>) -> impl Responder {
    println!("--- Get Open Orders ---");
    println!("Symbol: {}, Side: {}", info.symbol, info.side);

    // TODO: Fetch open orders from model

    let orders = vec![
        OrderDto {
            order_id: "ord-1".to_string(),
            symbol: info.symbol.clone(),
            side: info.side.clone(),
            quantity: Decimal::new(10, 0),
            remaining_quantity: Decimal::new(10, 0),
            price: Some(Decimal::new(150, 0)),
            tif: "GTC".to_string(),
            timestamp: 1234567890,
            type_: "LIMIT".to_string(),
        }
    ];

    println!("Open orders count: {}", orders.len());
    HttpResponse::Ok().json(orders)
}

pub async fn reset_book(req: web::Json<ResetBookRequest>) -> impl Responder {
    println!("--- Reset Book ---");
    println!("Symbol: {}", req.symbol);

    // TODO: Call model to reset book

    println!("Book reset for symbol: {}", req.symbol);
    HttpResponse::Ok().finish()
}

pub async fn get_recent_trades(info: web::Query<GetRecentTradesRequest>) -> impl Responder {
    println!("--- Get Recent Trades ---");
    println!("Symbol: {}, Limit: {}", info.symbol, info.limit);

    // TODO: Fetch recent trades from model

    let trades = vec![
        TradeDto {
            trade_id: "trade-recent-1".to_string(),
            symbol: info.symbol.clone(),
            price: Decimal::new(100, 0),
            quantity: Decimal::new(5, 0),
            taker_order_id: "taker-1".to_string(),
            maker_order_id: "maker-1".to_string(),
            timestamp: 1234567890,
        }
    ];

    println!("Recent trades returned: {}", trades.len());
    HttpResponse::Ok().json(trades)
}

pub async fn export_trades(req: web::Json<ExportRequest>) -> impl Responder {
    println!("--- Export Trades ---");
    println!("Symbol: {}, Path: {}", req.symbol, req.path);

    // TODO: Call model to export trades

    println!("Trades exported to: {}", req.path);
    HttpResponse::Ok().finish()
}

pub async fn export_book_history(req: web::Json<ExportRequest>) -> impl Responder {
    println!("--- Export Book History ---");
    println!("Symbol: {}, Path: {}", req.symbol, req.path);

    // TODO: Call model to export book history

    println!("Book history exported to: {}", req.path);
    HttpResponse::Ok().finish()
}