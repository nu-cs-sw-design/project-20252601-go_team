use actix_web::{web, HttpResponse, Responder};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::model::{
    LimitOrder, MarketOrder, Order, OrderBook, Side, Stock, TimeInForce, TradableAsset, Trade,
};

// Global registry of order books keyed by symbol.
static ORDER_BOOKS: OnceLock<Mutex<HashMap<String, OrderBook>>> = OnceLock::new();

fn order_books() -> &'static Mutex<HashMap<String, OrderBook>> {
    ORDER_BOOKS.get_or_init(|| Mutex::new(HashMap::new()))
}

// --- DTOs for Responses ---

#[derive(Debug, Serialize, Deserialize)]
pub struct TradeDto {
    pub trade_id: String,
    pub symbol: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub taker_order_id: String,
    pub maker_order_id: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaceLimitOrderResponse {
    pub order_id: String,
    pub trades: Vec<TradeDto>,
    pub resting_order: Option<OrderDto>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaceMarketOrderResponse {
    pub order_id: String,
    pub trades: Vec<TradeDto>,
    pub executed_quantity: Decimal,
    pub fully_filled: bool,
    pub tif: String,
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
    let side = match parse_side(&req.side) {
        Some(side) => side,
        None => return HttpResponse::BadRequest().body("Invalid side. Must be BUY or SELL"),
    };

    // 2. Parse TimeInForce Enum
    let tif = match parse_tif(&req.tif) {
        Some(tif) => tif,
        None => {
            return HttpResponse::BadRequest().body("Invalid TIF. Must be GTC, IOC, or FOK");
        }
    };

    if req.quantity <= Decimal::ZERO {
        return HttpResponse::BadRequest().body("Quantity must be positive");
    }

    if req.price <= Decimal::ZERO {
        return HttpResponse::BadRequest().body("Price must be positive");
    }

    // 3. Create Asset (Mocking lookup since we don't have the simulator state here)
    let asset: Arc<dyn TradableAsset> = Arc::new(Stock::new(
        String::from(&req.symbol),
        String::from("Unknown Name"),
        String::from("Created via API"),
    ));

    // 4. Generate Metadata
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let order_id = format!("ord-{}-{}", req.symbol, timestamp);

    // 5. Instantiate the actual Domain Object
    let order = LimitOrder::new(
        order_id.clone(),
        asset.clone(),
        side,
        req.quantity,
        timestamp,
        tif,
        req.price,
    );

    println!("Successfully created internal order object:");
    println!("{:#?}", order);

    let books = order_books();
    let mut books_guard = match books.lock() {
        Ok(guard) => guard,
        Err(err) => {
            eprintln!("Failed to lock order book registry: {}", err);
            return HttpResponse::InternalServerError().body("Order book unavailable");
        }
    };

    let symbol_key = req.symbol.clone();
    let book = books_guard
        .entry(symbol_key.clone())
        .or_insert_with(|| OrderBook::new(asset.clone()));

    let trades = book.add_limit_order(order);
    let resting_order = book.get_order(&order_id).map(limit_order_to_dto);

    drop(books_guard);

    let trade_dtos: Vec<TradeDto> = trades.into_iter().map(trade_to_dto).collect();

    let response = PlaceLimitOrderResponse {
        order_id,
        trades: trade_dtos,
        resting_order,
    };

    HttpResponse::Ok().json(response)
}

pub async fn place_market_order(req: web::Json<PlaceMarketOrderRequest>) -> impl Responder {
    println!("--- Placing Market Order ---");
    println!(
        "Symbol: {}, Side: {}, Qty: {}, TIF: {}",
        req.symbol, req.side, req.quantity, req.tif
    );

    let side = match parse_side(&req.side) {
        Some(side) => side,
        None => return HttpResponse::BadRequest().body("Invalid side. Must be BUY or SELL"),
    };

    if req.quantity <= Decimal::ZERO {
        return HttpResponse::BadRequest().body("Quantity must be positive");
    }

    let tif = match parse_tif(&req.tif) {
        Some(TimeInForce::GTC) => {
            println!("Market order received with GTC TIF; coercing to IOC");
            TimeInForce::IOC
        }
        Some(tif) => tif,
        None => return HttpResponse::BadRequest().body("Invalid TIF. Must be GTC, IOC, or FOK"),
    };

    let asset: Arc<dyn TradableAsset> = Arc::new(Stock::new(
        String::from(&req.symbol),
        String::from("Unknown Name"),
        String::from("Created via API"),
    ));

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let order_id = format!("mkt-{}-{}", req.symbol, timestamp);

    let market_order = MarketOrder::new(
        order_id.clone(),
        asset.clone(),
        side,
        req.quantity,
        timestamp,
        tif,
    );

    let books = order_books();
    let mut books_guard = match books.lock() {
        Ok(guard) => guard,
        Err(err) => {
            eprintln!("Failed to lock order book registry: {}", err);
            return HttpResponse::InternalServerError().body("Order book unavailable");
        }
    };

    let book = books_guard
        .entry(req.symbol.clone())
        .or_insert_with(|| OrderBook::new(asset.clone()));

    let trades = book.add_market_order(market_order);
    let executed_quantity = trades
        .iter()
        .fold(Decimal::ZERO, |acc, trade| acc + trade.quantity);

    drop(books_guard);

    let trade_dtos: Vec<TradeDto> = trades.into_iter().map(trade_to_dto).collect();

    let response = PlaceMarketOrderResponse {
        order_id,
        trades: trade_dtos,
        executed_quantity,
        fully_filled: executed_quantity >= req.quantity,
        tif: tif_to_string(tif).to_string(),
    };

    println!(
        "Market Order placed. Trades generated: {}, Executed Qty: {}",
        response.trades.len(),
        response.executed_quantity
    );
    HttpResponse::Ok().json(response)
}

pub async fn cancel_order(req: web::Json<CancelOrderRequest>) -> impl Responder {
    println!("--- Cancelling Order ---");
    println!("Symbol: {}, OrderID: {}", req.symbol, req.order_id);
    if req.symbol.trim().is_empty() || req.order_id.trim().is_empty() {
        return HttpResponse::BadRequest().body("Symbol and order_id must be provided");
    }

    let books = order_books();
    let mut books_guard = match books.lock() {
        Ok(guard) => guard,
        Err(err) => {
            eprintln!("Failed to lock order book registry: {}", err);
            return HttpResponse::InternalServerError().body("Order book unavailable");
        }
    };

    let success = books_guard
        .get_mut(&req.symbol)
        .map(|book| book.cancel_order(&req.order_id))
        .unwrap_or(false);

    println!("Order cancelled status: {}", success);
    HttpResponse::Ok().json(success)
}

pub async fn modify_order(req: web::Json<ModifyOrderRequest>) -> impl Responder {
    println!("--- Modifying Order ---");
    println!(
        "Symbol: {}, OrderID: {}, NewPrice: {}, NewQty: {}",
        req.symbol, req.order_id, req.new_price, req.new_quantity
    );

    if req.symbol.trim().is_empty() || req.order_id.trim().is_empty() {
        return HttpResponse::BadRequest().body("Symbol and order_id must be provided");
    }

    if req.new_price <= Decimal::ZERO {
        return HttpResponse::BadRequest().body("New price must be positive");
    }

    if req.new_quantity <= Decimal::ZERO {
        return HttpResponse::BadRequest().body("New quantity must be positive");
    }

    let books = order_books();
    let mut books_guard = match books.lock() {
        Ok(guard) => guard,
        Err(err) => {
            eprintln!("Failed to lock order book registry: {}", err);
            return HttpResponse::InternalServerError().body("Order book unavailable");
        }
    };

    let success = books_guard
        .get_mut(&req.symbol)
        .map(|book| book.modify_order(&req.order_id, req.new_price, req.new_quantity))
        .unwrap_or(false);

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

    let orders = vec![OrderDto {
        order_id: "ord-1".to_string(),
        symbol: info.symbol.clone(),
        side: info.side.clone(),
        quantity: Decimal::new(10, 0),
        remaining_quantity: Decimal::new(10, 0),
        price: Some(Decimal::new(150, 0)),
        tif: "GTC".to_string(),
        timestamp: 1234567890,
        type_: "LIMIT".to_string(),
    }];

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

    let trades = vec![TradeDto {
        trade_id: "trade-recent-1".to_string(),
        symbol: info.symbol.clone(),
        price: Decimal::new(100, 0),
        quantity: Decimal::new(5, 0),
        taker_order_id: "taker-1".to_string(),
        maker_order_id: "maker-1".to_string(),
        timestamp: 1234567890,
    }];

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

fn parse_side(input: &str) -> Option<Side> {
    match input.trim().to_ascii_uppercase().as_str() {
        "BUY" => Some(Side::BUY),
        "SELL" => Some(Side::SELL),
        _ => None,
    }
}

fn parse_tif(input: &str) -> Option<TimeInForce> {
    match input.trim().to_ascii_uppercase().as_str() {
        "GTC" => Some(TimeInForce::GTC),
        "IOC" => Some(TimeInForce::IOC),
        "FOK" => Some(TimeInForce::FOK),
        _ => None,
    }
}

fn side_to_string(side: Side) -> &'static str {
    match side {
        Side::BUY => "BUY",
        Side::SELL => "SELL",
    }
}

fn tif_to_string(tif: TimeInForce) -> &'static str {
    match tif {
        TimeInForce::GTC => "GTC",
        TimeInForce::IOC => "IOC",
        TimeInForce::FOK => "FOK",
    }
}

fn limit_order_to_dto(order: &LimitOrder) -> OrderDto {
    OrderDto {
        order_id: order.order_id().to_string(),
        symbol: order.asset().ticker().to_string(),
        side: side_to_string(order.side()).to_string(),
        quantity: order.quantity(),
        remaining_quantity: order.remaining_quantity(),
        price: Some(order.price()),
        tif: tif_to_string(order.tif()).to_string(),
        timestamp: order.timestamp(),
        type_: "LIMIT".to_string(),
    }
}

fn trade_to_dto(trade: Trade) -> TradeDto {
    let Trade {
        trade_id,
        asset,
        price,
        quantity,
        taker_order_id,
        maker_order_id,
        timestamp,
    } = trade;

    let symbol = asset.ticker().to_string();

    TradeDto {
        trade_id,
        symbol,
        price,
        quantity,
        taker_order_id,
        maker_order_id,
        timestamp,
    }
}
