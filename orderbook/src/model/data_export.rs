// src/model/data_export.rs

use std::fs::File;
use std::io::{self, Write};

use super::{OrderBookSnapshot, Trade};

/// Interface to export trades and book history.
pub trait DataExporter: Send + Sync {
    fn export_trades(&self, trades: &[Trade], path: &str) -> io::Result<()>;
    fn export_book_history(&self, history: &[OrderBookSnapshot], path: &str) -> io::Result<()>;
}

/// Simple CSV exporter implementation.
#[derive(Debug, Default)]
pub struct CsvDataExporter;

impl CsvDataExporter {
    pub fn new() -> Self {
        Self::default()
    }
}

impl DataExporter for CsvDataExporter {
    fn export_trades(&self, trades: &[Trade], path: &str) -> io::Result<()> {
        let mut file = File::create(path)?;
        // Header
        writeln!(
            file,
            "trade_id,symbol,price,quantity,taker_order_id,maker_order_id,timestamp"
        )?;

        for t in trades {
            writeln!(
                file,
                "{},{},{},{},{},{},{}",
                t.trade_id,
                t.asset.ticker(),
                t.price,
                t.quantity,
                t.taker_order_id,
                t.maker_order_id,
                t.timestamp
            )?;
        }

        Ok(())
    }

    fn export_book_history(&self, history: &[OrderBookSnapshot], path: &str) -> io::Result<()> {
        let mut file = File::create(path)?;
        // Very simple summary: one row per snapshot
        writeln!(file, "timestamp,symbol,bid_levels,ask_levels,recent_trades")?;

        for snap in history {
            writeln!(
                file,
                "{},{},{},{},{}",
                snap.timestamp,
                snap.symbol,
                snap.bids.len(),
                snap.asks.len(),
                snap.recent_trades.len()
            )?;
        }

        Ok(())
    }
}
