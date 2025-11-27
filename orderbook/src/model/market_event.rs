// src/model/market_event.rs

use std::fmt::Debug;
use rust_decimal::Decimal;

use super::{Side, TimeInForce};

/// High-level event type used to drive simulations.
#[derive(Debug, Clone)]
pub enum MarketEvent {
    Add(AddOrderEvent),
    Cancel(CancelOrderEvent),
    Modify(ModifyOrderEvent),
}

/// An event that adds a new order (limit or market) to a book.
#[derive(Debug, Clone)]
pub struct AddOrderEvent {
    pub timestamp: i64,
    pub symbol: String,
    pub side: Side,
    pub quantity: Decimal,
    pub tif: TimeInForce,
    pub kind: AddOrderKind,
}

/// Whether this add-event is for a limit or market order.
#[derive(Debug, Clone)]
pub enum AddOrderKind {
    Limit { price: Decimal },
    Market,
}

/// Cancel a resting order by ID.
#[derive(Debug, Clone)]
pub struct CancelOrderEvent {
    pub timestamp: i64,
    pub symbol: String,
    pub order_id: String,
}

/// Modify a resting limit order.
#[derive(Debug, Clone)]
pub struct ModifyOrderEvent {
    pub timestamp: i64,
    pub symbol: String,
    pub order_id: String,
    pub new_price: Decimal,
    pub new_quantity: Decimal,
}

/// Interface to load a stream of market events from a file (CSV, JSON, etc.)
pub trait EventLoader: Send + Sync {
    fn load_events(&self, path: &str) -> std::io::Result<Vec<MarketEvent>>;
}
