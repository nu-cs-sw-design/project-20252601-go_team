# TradingSimulator API Documentation

## Overview
This document describes the API methods added to the `TradingSimulator` class in the Controller package. These methods provide a comprehensive interface for interacting with OrderBooks, managing simulations, and accessing trading data.

## API Categories

### 1. Order Management API
Methods for placing, canceling, and modifying orders.

#### `placeOrder(symbol: String, order: Order): List<Trade>`
Places a new order in the specified OrderBook.
- **Parameters:**
  - `symbol`: The ticker symbol of the asset (e.g., "AAPL", "BTC")
  - `order`: The order to place (LimitOrder or MarketOrder)
- **Returns:** List of trades executed as a result of placing this order
- **Use Case:** User submits a new buy or sell order through the UI

#### `cancelOrder(symbol: String, orderId: String): boolean`
Cancels an existing order in the OrderBook.
- **Parameters:**
  - `symbol`: The ticker symbol of the asset
  - `orderId`: Unique identifier of the order to cancel
- **Returns:** `true` if the order was successfully canceled, `false` otherwise
- **Use Case:** User cancels a pending order

#### `modifyOrder(symbol: String, orderId: String, newPrice: Decimal, newQty: Decimal): boolean`
Modifies an existing GTC LimitOrder's price and/or quantity.
- **Parameters:**
  - `symbol`: The ticker symbol of the asset
  - `orderId`: Unique identifier of the order to modify
  - `newPrice`: New price for the order
  - `newQty`: New quantity for the order
- **Returns:** `true` if the order was successfully modified, `false` otherwise
- **Note:** Only applies to GTC LimitOrders
- **Use Case:** User adjusts their pending limit order

### 2. Order Book Query API
Methods for retrieving current OrderBook state and market data.

#### `getOrderBook(symbol: String): OrderBook`
Retrieves the OrderBook instance for a specific symbol.
- **Parameters:**
  - `symbol`: The ticker symbol of the asset
- **Returns:** The OrderBook instance
- **Use Case:** Direct access to OrderBook for advanced operations

#### `getBestBid(symbol: String): PriceLevelInfo`
Gets the highest bid price level in the OrderBook.
- **Parameters:**
  - `symbol`: The ticker symbol of the asset
- **Returns:** PriceLevelInfo with the best bid price and total volume
- **Use Case:** Display current best bid in the UI

#### `getBestAsk(symbol: String): PriceLevelInfo`
Gets the lowest ask price level in the OrderBook.
- **Parameters:**
  - `symbol`: The ticker symbol of the asset
- **Returns:** PriceLevelInfo with the best ask price and total volume
- **Use Case:** Display current best ask in the UI

#### `getTopOfBook(symbol: String): TopOfBook`
Gets both best bid and best ask with calculated spread.
- **Parameters:**
  - `symbol`: The ticker symbol of the asset
- **Returns:** TopOfBook containing bestBid, bestAsk, and spread
- **Use Case:** Display market spread in the UI

#### `getDepth(symbol: String, levels: int): OrderBookDepth`
Gets multiple price levels from both bid and ask sides.
- **Parameters:**
  - `symbol`: The ticker symbol of the asset
  - `levels`: Number of price levels to retrieve from each side
- **Returns:** OrderBookDepth with lists of bid and ask price levels
- **Use Case:** Display order book depth in the OrderBookView

#### `getSnapshot(symbol: String): OrderBookSnapshot`
Gets a complete snapshot of the current OrderBook state.
- **Parameters:**
  - `symbol`: The ticker symbol of the asset
- **Returns:** OrderBookSnapshot with timestamp, bids, asks, and recent trades
- **Use Case:** Update UI components, data export, or historical tracking

### 3. Order Query API
Methods for querying individual orders and order lists.

#### `getOrder(symbol: String, orderId: String): Order`
Retrieves a specific order by its ID.
- **Parameters:**
  - `symbol`: The ticker symbol of the asset
  - `orderId`: Unique identifier of the order
- **Returns:** The Order object if found, null otherwise
- **Use Case:** Display order details or verify order status

#### `getOpenOrders(symbol: String, side: Side): List<Order>`
Gets all open orders for a specific side (BUY or SELL).
- **Parameters:**
  - `symbol`: The ticker symbol of the asset
  - `side`: Side.BUY or Side.SELL
- **Returns:** List of all open orders on the specified side
- **Use Case:** Display all pending buy or sell orders

### 4. Trade History API
Methods for accessing executed trade data.

#### `getRecentTrades(symbol: String, limit: int): List<Trade>`
Gets the most recent trades up to a specified limit.
- **Parameters:**
  - `symbol`: The ticker symbol of the asset
  - `limit`: Maximum number of trades to return
- **Returns:** List of recent Trade objects, ordered by timestamp (newest first)
- **Use Case:** Display recent trade activity in TradeHistoryView

#### `getAllTrades(symbol: String): List<Trade>`
Gets all trades in the OrderBook's history.
- **Parameters:**
  - `symbol`: The ticker symbol of the asset
- **Returns:** Complete list of all trades
- **Use Case:** Full trade history export or analysis

### 5. Observer Management API
Methods for registering and unregistering observers for real-time updates.

#### `addOrderBookObserver(symbol: String, observer: OrderBookObserver): void`
Registers an observer to receive OrderBook updates.
- **Parameters:**
  - `symbol`: The ticker symbol of the asset
  - `observer`: Observer instance (typically a View component like OrderBookView)
- **Use Case:** Connect UI components to receive real-time updates

#### `removeOrderBookObserver(symbol: String, observer: OrderBookObserver): void`
Unregisters an observer from receiving OrderBook updates.
- **Parameters:**
  - `symbol`: The ticker symbol of the asset
  - `observer`: Observer instance to remove
- **Use Case:** Cleanup when a view component is closed or destroyed

### 6. OrderBook Lifecycle API
Methods for creating and managing OrderBook instances.

#### `createOrderBook(asset: TradableAsset): OrderBook`
Creates a new OrderBook for a specific asset.
- **Parameters:**
  - `asset`: The TradableAsset (Stock or Crypto) to create an OrderBook for
- **Returns:** The newly created OrderBook instance
- **Use Case:** Initialize trading for a new asset

#### `getAvailableSymbols(): List<String>`
Gets a list of all available trading symbols.
- **Returns:** List of ticker symbols for all active OrderBooks
- **Use Case:** Populate asset selection dropdown in the UI

#### `resetOrderBook(symbol: String): void`
Resets an OrderBook to its initial empty state.
- **Parameters:**
  - `symbol`: The ticker symbol of the asset
- **Use Case:** Clear all orders and trades, typically for simulation reset

### 7. Simulation API
Methods for managing simulation sessions.

#### `createSimulation(sessionId: String, symbol: String, eventsFile: String): SimulationSession`
Creates a new simulation session from an events file.
- **Parameters:**
  - `sessionId`: Unique identifier for this simulation session
  - `symbol`: The ticker symbol to simulate
  - `eventsFile`: Path to the file containing market events
- **Returns:** The created SimulationSession instance
- **Use Case:** Start a new simulation from historical events

#### `getSimulation(sessionId: String): SimulationSession`
Retrieves an existing simulation session.
- **Parameters:**
  - `sessionId`: Unique identifier of the simulation session
- **Returns:** The SimulationSession instance if found, null otherwise
- **Use Case:** Access simulation for stepping or status checking

#### `removeSimulation(sessionId: String): void`
Removes and cleans up a simulation session.
- **Parameters:**
  - `sessionId`: Unique identifier of the simulation session
- **Use Case:** Cleanup after simulation completion

### 8. Data Export API
Methods for exporting trading data.

#### `exportTrades(symbol: String, path: String): void`
Exports all trades for a symbol to a file.
- **Parameters:**
  - `symbol`: The ticker symbol of the asset
  - `path`: File path for the export (typically CSV)
- **Use Case:** Export trade history for analysis

#### `exportBookHistory(symbol: String, path: String): void`
Exports OrderBook snapshot history to a file.
- **Parameters:**
  - `symbol`: The ticker symbol of the asset
  - `path`: File path for the export (typically CSV)
- **Use Case:** Export OrderBook state changes over time

## Design Rationale

### Symbol-Based Access
All API methods take a `symbol` parameter to identify which OrderBook to operate on. This design:
- Supports multiple concurrent OrderBooks for different assets
- Makes the API clear and explicit about which asset is being accessed
- Enables multi-asset trading interfaces

### Delegation Pattern
The TradingSimulator acts as a facade, delegating calls to the appropriate OrderBook instance. Benefits:
- **Centralized Control**: All trading operations go through one controller
- **Security**: The simulator can add validation, logging, or access control
- **Flexibility**: Can add cross-OrderBook operations in the future
- **Simplicity**: View components only need a reference to TradingSimulator, not individual OrderBooks

### Comprehensive API Coverage
The API covers all necessary operations for:
- **Trading**: Place, cancel, modify orders
- **Monitoring**: Query order book state, orders, and trades
- **Real-time Updates**: Observer pattern integration
- **Simulation**: Load and run event-based simulations
- **Data Management**: Export and persistence

## Integration with View Package

### OrderBookView Integration
```
OrderBookView → TradingSimulator.addOrderBookObserver()
OrderBookView implements OrderBookObserver
OrderBook → notifies → OrderBookView.onOrderBookUpdate()
```

### ControlPanel Integration
```
ControlPanel → TradingSimulator.placeOrder()
ControlPanel → TradingSimulator.cancelOrder()
ControlPanel → TradingSimulator.createSimulation()
ControlPanel → SimulationSession.step() / runToEnd()
```

### TradeHistoryView Integration
```
TradeHistoryView → TradingSimulator.getRecentTrades()
TradeHistoryView ← receives updates via OrderBookObserver.onNewTrade()
```

### ChartView Integration
```
ChartView → TradingSimulator.getSnapshot()
ChartView ← receives updates via OrderBookObserver callbacks
```

## Usage Examples

### Example 1: Placing a Limit Order
```
LimitOrder order = new LimitOrder(...)
order.asset = stock
order.side = Side.BUY
order.quantity = 100
order.price = 50.25

List<Trade> trades = simulator.placeOrder("AAPL", order)
// Process any immediately executed trades
```

### Example 2: Monitoring Order Book
```
// Get top of book for display
TopOfBook tob = simulator.getTopOfBook("AAPL")
display.showBid(tob.bestBid)
display.showAsk(tob.bestAsk)
display.showSpread(tob.spread)

// Get market depth
OrderBookDepth depth = simulator.getDepth("AAPL", 10)
orderBookView.renderBids(depth.bids)
orderBookView.renderAsks(depth.asks)
```

### Example 3: Setting Up Real-time Updates
```
// View component registers as observer
OrderBookView view = new OrderBookView()
simulator.addOrderBookObserver("AAPL", view)

// View receives automatic callbacks
view.onOrderBookUpdate(snapshot)  // When book changes
view.onNewTrade(trade)            // When trades execute
```

### Example 4: Running a Simulation
```
// Create simulation from events file
SimulationSession session = simulator.createSimulation(
    "sim-001", 
    "AAPL", 
    "data/events.csv"
)

// Step through events
session.step(10)  // Process 10 events

// Run remaining events
session.runToEnd()

// Export results
simulator.exportTrades("AAPL", "output/trades.csv")
```

## Performance Considerations

### Efficient Query Methods
- `getBestBid/Ask()`: O(1) - Accesses sorted map's first entry
- `getDepth(n)`: O(n) - Returns top n levels
- `getOrder(id)`: O(1) - Hash map lookup
- `getRecentTrades(limit)`: O(limit) - Returns tail of trade history

### Observer Pattern
- Avoids polling overhead
- Updates only when state changes
- Supports multiple observers per OrderBook

### Symbol-Based Routing
- O(1) OrderBook lookup via hash map
- Scales to many concurrent assets

## Alignment with Requirements

Based on `ranking.md`:

1. **Functional Correctness**: All API methods delegate to tested OrderBook operations
2. **Time Behavior (20ms trades)**: Direct delegation ensures minimal overhead
3. **Scalability (100 req/sec)**: Efficient data structures support high throughput
4. **Time Behavior (500ms GUI)**: Observer pattern enables immediate UI updates
5. **Learnability**: Clear, well-documented API methods with intuitive names

## Summary

The TradingSimulator API provides a complete, well-organized interface for:
- ✅ Order management (place, cancel, modify)
- ✅ Market data queries (prices, depth, snapshots)
- ✅ Order tracking (individual orders, open orders)
- ✅ Trade history access
- ✅ Real-time update subscriptions
- ✅ OrderBook lifecycle management
- ✅ Simulation control
- ✅ Data export

This API enables the View package to provide a fully-featured trading interface while maintaining clean separation between Controller and View layers.
