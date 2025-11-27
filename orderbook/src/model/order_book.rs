// src/model/order_book.rs

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use rust_decimal::Decimal;

use super::{
    TradableAsset,
    Side,
    TimeInForce,
    LimitOrder,
    MarketOrder,
    Order,
    Trade,
    PriceLevel,
};

// Lightweight reference to locate an order in the book by ID
#[derive(Debug, Clone)]
struct OrderRef {
    side: Side,
    price: Decimal,
}

// Read-only info about a price level (for views / API)
#[derive(Debug, Clone)]
pub struct PriceLevelInfo {
    pub price: Decimal,
    pub total_volume: Decimal,
}

// Read-only depth snapshot (top N levels)
#[derive(Debug, Clone)]
pub struct OrderBookDepth {
    pub bids: Vec<PriceLevelInfo>,
    pub asks: Vec<PriceLevelInfo>,
}

// Full-ish snapshot of the book at a moment in time
#[derive(Debug, Clone)]
pub struct OrderBookSnapshot {
    pub symbol: String,
    pub timestamp: i64,
    pub bids: Vec<PriceLevelInfo>,
    pub asks: Vec<PriceLevelInfo>,
    pub recent_trades: Vec<Trade>,
}

// Top-of-book view: best bid, best ask, and spread
#[derive(Debug, Clone)]
pub struct TopOfBook {
    pub best_bid: Option<PriceLevelInfo>,
    pub best_ask: Option<PriceLevelInfo>,
    pub spread: Option<Decimal>, // ask - bid, if both sides exist
}

// Observer interface for order book updates
pub trait OrderBookObserver: Send + Sync {
    fn on_order_book_update(&self, snapshot: &OrderBookSnapshot);
    fn on_new_trade(&self, trade: &Trade);
}


// Core order book for a single asset
pub struct OrderBook {
    asset: Arc<dyn TradableAsset>,

    // Bids: sorted ascending by price; best bid = highest key
    bids: BTreeMap<Decimal, PriceLevel>,

    // Asks: sorted ascending by price; best ask = lowest key
    asks: BTreeMap<Decimal, PriceLevel>,

    // Map orderId -> side + price, to locate orders later
    orders_by_id: HashMap<String, OrderRef>,

    // Simple in-memory trade history
    trade_history: Vec<Trade>,

    // Observers subscribed for updates
    observers: Vec<Arc<dyn OrderBookObserver>>,
}

impl OrderBook {
    // Create a new empty book for the given asset
    pub fn new(asset: Arc<dyn TradableAsset>) -> Self {
        Self {
            asset,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            orders_by_id: HashMap::new(),
            trade_history: Vec::new(),
            observers: Vec::new(),
        }
    }

    // --- Observer management ---
    pub fn add_observer(&mut self, observer: Arc<dyn OrderBookObserver>) {
        self.observers.push(observer);
    }

    pub fn remove_observer(&mut self, observer: &Arc<dyn OrderBookObserver>) {
        self.observers
            .retain(|obs| !Arc::ptr_eq(obs, observer));
    }



    // Add a limit order: match against opposite side first, then rest any remaining GTC quantity on this book
    pub fn add_limit_order(&mut self, mut order: LimitOrder) -> Vec<Trade> {
        let mut trades = Vec::new();

        match order.side() {
            Side::BUY => {
                self.match_against_asks_for_limit(&mut order, &mut trades);
            }
            Side::SELL => {
                self.match_against_bids_for_limit(&mut order, &mut trades);
            }
        }

        // For now: only GTC orders rest on the book
        if !order.is_filled() && matches!(order.tif(), TimeInForce::GTC) {
            self.insert_resting_limit(order);
        }

        self.trade_history.extend(trades.iter().cloned());

        for t in &trades {
            self.notify_new_trade(t);
        }
        self.notify_book_update();

        trades
    }

    // Add a market order: match against opposite side, do not rest
    pub fn add_market_order(&mut self, mut order: MarketOrder) -> Vec<Trade> {
        let mut trades = Vec::new();

        match order.side() {
            Side::BUY => {
                self.match_against_asks_for_market(&mut order, &mut trades);
            }
            Side::SELL => {
                self.match_against_bids_for_market(&mut order, &mut trades);
            }
        }

        self.trade_history.extend(trades.iter().cloned());

        for t in &trades {
            self.notify_new_trade(t);
        }
        self.notify_book_update();
        
        trades
    }

    // Cancel an order by ID - Only applies to resting GTC limit orders
    pub fn cancel_order(&mut self, order_id: &str) -> bool {
        let order_ref = match self.orders_by_id.remove(order_id) {
            Some(r) => r,
            None => return false,
        };

        let book = match order_ref.side {
            Side::BUY => &mut self.bids,
            Side::SELL => &mut self.asks,
        };

        let success = if let Some(level) = book.get_mut(&order_ref.price) {
            let removed = level.remove_order(order_id);
            if level.total_volume() == Decimal::ZERO {
                book.remove(&order_ref.price);
            }
            removed
        } else {
            false
        };

        if success {
            self.notify_book_update();
        }

        success
    }

    // Modify an existing GTC limit order's price and quantity
    /*
    For simplicity, this:
        - cancels the old resting order
        - re-inserts a new LimitOrder with same ID but new price/qty
    */
    pub fn modify_order(
        &mut self,
        order_id: &str,
        new_price: Decimal,
        new_qty: Decimal,
    ) -> bool {
        // Look up location
        let order_ref = match self.orders_by_id.get(order_id).cloned() {
            Some(r) => r,
            None => return false,
        };

        let book = match order_ref.side {
            Side::BUY => &mut self.bids,
            Side::SELL => &mut self.asks,
        };

        // Take the existing LimitOrder out of its level
        let existing = {
            let level = match book.get_mut(&order_ref.price) {
                Some(l) => l,
                None => return false,
            };
            level.take_order(order_id)
        };

        let mut existing = match existing {
            Some(o) => o,
            None => return false,
        };

        // Only allow modifying GTC resting orders
        if !matches!(existing.tif(), TimeInForce::GTC) {
            // Put it back to avoid corrupting state?
            // For simplicity just treat this as a failed modify
            return false;
        }

        // Build a new LimitOrder with the same ID and metadata but new price/qty
        let new_order = LimitOrder::new(
            existing.order_id().to_string(),
            existing.clone_asset(),
            existing.side(),
            new_qty,
            existing.timestamp(),
            existing.tif(),
            new_price,
        );

        // Clean up empty level if needed
        if let Some(level) = book.get(&order_ref.price) {
            if level.total_volume() == Decimal::ZERO {
                book.remove(&order_ref.price);
            }
        }

        // Update orders_by_id to new price
        self.orders_by_id.insert(
            order_id.to_string(),
            OrderRef {
                side: order_ref.side,
                price: new_price,
            },
        );

        // Insert the modified order as if it were a fresh one
        self.insert_resting_limit(new_order);
        self.notify_book_update();

        true
    }

    // Get a reference to a resting limit order by ID (if present)
    pub fn get_order(&self, order_id: &str) -> Option<&LimitOrder> {
        let order_ref = self.orders_by_id.get(order_id)?;
        let book = match order_ref.side {
            Side::BUY => &self.bids,
            Side::SELL => &self.asks,
        };
        let level = book.get(&order_ref.price)?;
        level.orders().find(|o| o.order_id() == order_id)
    }

    // Get all open resting limit orders for a given side
    pub fn get_open_orders(&self, side: Side) -> Vec<&LimitOrder> {
        let book = match side {
            Side::BUY => &self.bids,
            Side::SELL => &self.asks,
        };

        let mut out = Vec::new();
        for (_price, level) in book.iter() {
            for order in level.orders() {
                out.push(order);
            }
        }

        out
    }

    // Get best bid level (highest bid price)
    pub fn get_best_bid(&self) -> Option<PriceLevelInfo> {
        let (price, level) = self.bids.iter().next_back()?;
        Some(PriceLevelInfo {
            price: *price,
            total_volume: level.total_volume(),
        })
    }

    // Get best ask level (lowest ask price)
    pub fn get_best_ask(&self) -> Option<PriceLevelInfo> {
        let (price, level) = self.asks.iter().next()?;
        Some(PriceLevelInfo {
            price: *price,
            total_volume: level.total_volume(),
        })
    }

    // Get depth for top `levels` on both sides
    pub fn get_depth(&self, levels: usize) -> OrderBookDepth {
        let bids = self
            .bids
            .iter()
            .rev()
            .take(levels)
            .map(|(p, lvl)| PriceLevelInfo {
                price: *p,
                total_volume: lvl.total_volume(),
            })
            .collect();

        let asks = self
            .asks
            .iter()
            .take(levels)
            .map(|(p, lvl)| PriceLevelInfo {
                price: *p,
                total_volume: lvl.total_volume(),
            })
            .collect();

        OrderBookDepth { bids, asks }
    }

    // Get the most recent `limit` trades (from the end of history)
    pub fn get_recent_trades(&self, limit: usize) -> Vec<Trade> {
        let len = self.trade_history.len();
        let start = len.saturating_sub(limit);
        self.trade_history[start..].to_vec()
    }

    // Get a snapshot of the book
    // For now, timestamp is 0; will wire real time in later via events
    pub fn get_snapshot(&self) -> OrderBookSnapshot {
        let depth = self.get_depth(usize::MAX); // full depth

        OrderBookSnapshot {
            symbol: self.asset.ticker().to_string(),
            timestamp: 0,
            bids: depth.bids,
            asks: depth.asks,
            recent_trades: self.get_recent_trades(100), // arbitrary cap for snapshot
        }
    }

    pub fn get_top_of_book(&self) -> Option<TopOfBook> {
        let best_bid = self.get_best_bid();
        let best_ask = self.get_best_ask();

        if best_bid.is_none() && best_ask.is_none() {
            return None;
        }

        let spread = match (&best_bid, &best_ask) {
            (Some(bid), Some(ask)) => Some(ask.price - bid.price),
            _ => None,
        };

        Some(TopOfBook {
            best_bid,
            best_ask,
            spread,
        })
    }

    // Clear all book state and history
    pub fn reset(&mut self) {
        self.bids.clear();
        self.asks.clear();
        self.orders_by_id.clear();
        self.trade_history.clear();
        self.notify_book_update();
    }

    fn notify_book_update(&self) {
        if self.observers.is_empty() {
            return;
        }
        let snapshot = self.get_snapshot();
        for obs in &self.observers {
            obs.on_order_book_update(&snapshot);
        }
    }

    fn notify_new_trade(&self, trade: &Trade) {
        if self.observers.is_empty() {
            return;
        }
        for obs in &self.observers {
            obs.on_new_trade(trade);
        }
    }


    // --- Internal helpers ---

    fn insert_resting_limit(&mut self, order: LimitOrder) {
        let side = order.side();
        let price = order.price();
        let order_id = order.order_id().to_string();

        let book = match side {
            Side::BUY => &mut self.bids,
            Side::SELL => &mut self.asks,
        };

        let level = book.entry(price).or_insert_with(|| PriceLevel::new(price));
        level.add_order(order);

        self.orders_by_id.insert(order_id, OrderRef { side, price });
    }

    fn match_against_asks_for_limit(
        &mut self,
        taker: &mut LimitOrder,
        out_trades: &mut Vec<Trade>,
    ) {
        loop {
            if taker.is_filled() {
                break;
            }

            // Best ask = lowest ask price
            let best_ask_price = match self.asks.keys().next().cloned() {
                Some(p) => p,
                None => break,
            };

            // Stop if best ask is above taker's limit price
            if best_ask_price > taker.price() {
                break;
            }

            let trades = {
                let level = self.asks.get_mut(&best_ask_price).unwrap();
                level.match_with(taker as &mut dyn Order)
            };

            // Remove empty levels
            if let Some(level) = self.asks.get(&best_ask_price) {
                if level.total_volume() == Decimal::ZERO {
                    self.asks.remove(&best_ask_price);
                }
            }

            out_trades.extend(trades);
        }
    }

    fn match_against_bids_for_limit(
        &mut self,
        taker: &mut LimitOrder,
        out_trades: &mut Vec<Trade>,
    ) {
        loop {
            if taker.is_filled() {
                break;
            }

            // Best bid = highest bid price
            let best_bid_price = match self.bids.keys().next_back().cloned() {
                Some(p) => p,
                None => break,
            };

            // Stop if best bid is below taker's limit price
            if best_bid_price < taker.price() {
                break;
            }

            let trades = {
                let level = self.bids.get_mut(&best_bid_price).unwrap();
                level.match_with(taker as &mut dyn Order)
            };

            // Remove empty levels
            if let Some(level) = self.bids.get(&best_bid_price) {
                if level.total_volume() == Decimal::ZERO {
                    self.bids.remove(&best_bid_price);
                }
            }

            out_trades.extend(trades);
        }
    }

    fn match_against_asks_for_market(
        &mut self,
        taker: &mut MarketOrder,
        out_trades: &mut Vec<Trade>,
    ) {
        loop {
            if taker.is_filled() {
                break;
            }

            let best_ask_price = match self.asks.keys().next().cloned() {
                Some(p) => p,
                None => break,
            };

            let trades = {
                let level = self.asks.get_mut(&best_ask_price).unwrap();
                level.match_with(taker as &mut dyn Order)
            };

            if let Some(level) = self.asks.get(&best_ask_price) {
                if level.total_volume() == Decimal::ZERO {
                    self.asks.remove(&best_ask_price);
                }
            }

            out_trades.extend(trades);
        }
    }

    fn match_against_bids_for_market(
        &mut self,
        taker: &mut MarketOrder,
        out_trades: &mut Vec<Trade>,
    ) {
        loop {
            if taker.is_filled() {
                break;
            }

            let best_bid_price = match self.bids.keys().next_back().cloned() {
                Some(p) => p,
                None => break,
            };

            let trades = {
                let level = self.bids.get_mut(&best_bid_price).unwrap();
                level.match_with(taker as &mut dyn Order)
            };

            if let Some(level) = self.bids.get(&best_bid_price) {
                if level.total_volume() == Decimal::ZERO {
                    self.bids.remove(&best_bid_price);
                }
            }

            out_trades.extend(trades);
        }
    }
}
