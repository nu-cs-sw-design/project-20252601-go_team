// src/model/order.rs

use std::sync::Arc;
use rust_decimal::Decimal;
use super::{TradableAsset, Side, TimeInForce};

// Common behavior for all order types
pub trait Order {
    fn order_id(&self) -> &str;
    fn asset(&self) -> &dyn TradableAsset;
    fn side(&self) -> Side;
    fn quantity(&self) -> Decimal;
    fn remaining_quantity(&self) -> Decimal;
    fn timestamp(&self) -> i64;
    fn tif(&self) -> TimeInForce;
    fn is_filled(&self) -> bool { self.remaining_quantity().is_zero() }
    fn fill(&mut self, qty: Decimal); // Reduce remaining quantity when a trade executes
}

// A limit order with a specific price
#[derive(Debug)]
pub struct LimitOrder {
    order_id: String,
    // A shared, thread-safe pointer to a trait object implementing TradableAsset
    asset: Arc<dyn TradableAsset>,
    side: Side,
    quantity: Decimal,
    remaining_quantity: Decimal,
    timestamp: i64,
    tif: TimeInForce,
    price: Decimal,
}

// A market order - execute immediately at best available price
#[derive(Debug)]
pub struct MarketOrder {
    order_id: String,
    asset: Arc<dyn TradableAsset>,
    side: Side,
    quantity: Decimal,
    remaining_quantity: Decimal,
    timestamp: i64,
    tif: TimeInForce,
}

// Trade record
#[derive(Debug, Clone)]
pub struct Trade {
    pub trade_id: String,
    pub asset: Arc<dyn TradableAsset>,
    pub price: Decimal,
    pub quantity: Decimal,
    pub taker_order_id: String,
    pub maker_order_id: String,
    pub timestamp: i64,
}



//
/// Implementations:
//

impl LimitOrder {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        order_id: String,
        asset: Arc<dyn TradableAsset>,
        side: Side,
        quantity: Decimal,
        timestamp: i64,
        tif: TimeInForce,
        price: Decimal,
    ) -> Self {
        Self {
            remaining_quantity: quantity,
            order_id,
            asset,
            side,
            quantity,
            timestamp,
            tif,
            price,
        }
    }

    pub fn price(&self) -> Decimal {
        self.price
    }
}

impl Order for LimitOrder {
    fn order_id(&self) -> &str {
        &self.order_id
    }

    fn asset(&self) -> &dyn TradableAsset {
        self.asset.as_ref()
    }

    fn side(&self) -> Side {
        self.side
    }

    fn quantity(&self) -> Decimal {
        self.quantity
    }

    fn remaining_quantity(&self) -> Decimal {
        self.remaining_quantity
    }

    fn timestamp(&self) -> i64 {
        self.timestamp
    }

    fn tif(&self) -> TimeInForce {
        self.tif
    }

    fn fill(&mut self, qty: Decimal) {
        // Clamp at zero to avoid negative remaining quantity
        if qty >= self.remaining_quantity {
            self.remaining_quantity = Decimal::ZERO;
        } else {
            self.remaining_quantity -= qty;
        }
    }
}


impl MarketOrder {
    pub fn new(
        order_id: String,
        asset: Arc<dyn TradableAsset>,
        side: Side,
        quantity: Decimal,
        timestamp: i64,
        tif: TimeInForce,
    ) -> Self {
        Self {
            remaining_quantity: quantity,
            order_id,
            asset,
            side,
            quantity,
            timestamp,
            tif,
        }
    }
}

impl Order for MarketOrder {
    fn order_id(&self) -> &str {
        &self.order_id
    }

    fn asset(&self) -> &dyn TradableAsset {
        self.asset.as_ref()
    }

    fn side(&self) -> Side {
        self.side
    }

    fn quantity(&self) -> Decimal {
        self.quantity
    }

    fn remaining_quantity(&self) -> Decimal {
        self.remaining_quantity
    }

    fn timestamp(&self) -> i64 {
        self.timestamp
    }

    fn tif(&self) -> TimeInForce {
        self.tif
    }

    fn fill(&mut self, qty: Decimal) {
        if qty >= self.remaining_quantity {
            self.remaining_quantity = Decimal::ZERO;
        } else {
            self.remaining_quantity -= qty;
        }
    }
}


impl Trade {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        trade_id: String,
        asset: Arc<dyn TradableAsset>,
        price: Decimal,
        quantity: Decimal,
        taker_order_id: String,
        maker_order_id: String,
        timestamp: i64,
    ) -> Self {
        Self {
            trade_id,
            asset,
            price,
            quantity,
            taker_order_id,
            maker_order_id,
            timestamp,
        }
    }
}