// src/model/asset.rs

// Side of an Order
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    BUY, 
    SELL
}

// Time in force Policy for an order
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeInForce {
    GTC, // Good Till Cancelled
    IOC, // Immediate Or Cancel
    FOK // Fill Or Kill
}

// Any asset that can be traded - Thread Safe
pub trait TradableAsset: Send + Sync {
    fn ticker(&self) -> &str;
    fn name(&self)-> &str;
    fn description(&self)-> &str;
}

// A stock asset
#[derive(Debug, Clone)]
pub struct Stock {
    ticker: String,
    name: String,
    description: String,
}

// A crypto asset
#[derive(Debug, Clone)]
pub struct Crypto {
    ticker: String,
    name: String,
    description: String,
}



//
/// Implementations:
//

impl Stock {
    pub fn new<T: Into<String>>(ticker: T, name: T, description: T) -> Self {
        Self {
            ticker : ticker.into(),
            name : name.into(),
            description : description.into()
        }
    }
}

impl TradableAsset for Stock {
    fn ticker(&self) -> &str {
        &self.ticker
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }
}

impl Crypto {
    pub fn new<T: Into<String>>(ticker: T, name: T, description: T) -> Self {
        Self {
            ticker : ticker.into(),
            name : name.into(),
            description : description.into()
        }
    }
}

impl TradableAsset for Crypto {
    fn ticker(&self) -> &str {
        &self.ticker
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }
}