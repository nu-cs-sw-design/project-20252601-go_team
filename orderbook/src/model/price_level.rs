// src/model/price_level.rs

use std::collections::VecDeque;
use rust_decimal::Decimal;
use super::{LimitOrder, Order, Trade};

// A single price level in the order book
// Holds a FIFO queue of limit orders at the same price
#[derive(Debug)]
pub struct PriceLevel {
    price: Decimal,
    total_volume: Decimal,
    orders: VecDeque<LimitOrder>,
}

impl PriceLevel {
    // Create an empty price level at the given price
    pub fn new(price: Decimal) -> Self {
        Self {
            price,
            total_volume: Decimal::ZERO,
            orders: VecDeque::new(),
        }
    }

    /// Price of this level
    pub fn price(&self) -> Decimal {
        self.price
    }

    /// Total remaining volume at this price
    pub fn total_volume(&self) -> Decimal {
        self.total_volume
    }

    /// Add a new limit order to the back of the FIFO queue
    pub fn add_order(&mut self, order: LimitOrder) {
        self.total_volume += order.remaining_quantity();
        self.orders.push_back(order);
    }

    // Remove an order by ID, if present -> return true if removed
    pub fn remove_order(&mut self, order_id: &str) -> bool {
        if let Some(pos) = self
            .orders
            .iter()
            .position(|o| o.order_id() == order_id)
        {
            let removed = self.orders.remove(pos).unwrap();
            self.total_volume -= removed.remaining_quantity();
            true
        } else {
            false
        }
    }

    // Match a taker order against this price level's queue of makers -> returns the list of trades generated.
    // Assumes the caller has already decided that this price level is eligible for matching
    pub fn match_with(&mut self, taker: &mut dyn Order) -> Vec<Trade> {
        let mut trades = Vec::new();

        // While there is a taker quantity and makers in the queue
        while !taker.is_filled() {
            // Get the front maker order
            let maker = match self.orders.front_mut() {
                Some(m) => m,
                None => break, // No more makers at this level
            };

            // Skip already-filled makers just in case
            if maker.is_filled() {
                self.orders.pop_front();
                continue;
            }

            let maker_rem = maker.remaining_quantity();
            let taker_rem = taker.remaining_quantity();

            // Trade quantity = min(taker, maker)
            let trade_qty = if taker_rem <= maker_rem {
                taker_rem
            } else {
                maker_rem
            };

            // Update both orders remaining quantities.
            maker.fill(trade_qty);
            taker.fill(trade_qty);

            // Decrease total volume at this price.
            self.total_volume -= trade_qty;

            // Create a trade record.
            let trade = Trade::new(
                format!("{}-{}", taker.order_id(), maker.order_id()),
                taker.clone_asset(),
                self.price,
                trade_qty,
                taker.order_id().to_string(),
                maker.order_id().to_string(),
                taker.timestamp(), // for now, use taker's timestamp
            );
            trades.push(trade);

            // If maker fully filled, pop it from the queue.
            if maker.is_filled() {
                self.orders.pop_front();
            }
        }

        trades
    }

    /// Number of resting orders at this price
    pub fn order_count(&self) -> usize {
        self.orders.len()
    }
}
