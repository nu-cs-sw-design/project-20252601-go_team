# Multilevel Limit Orderbook - Architecture Overview

## System Architecture

This trading simulation system follows the **Model-View-Controller (MVC)** architectural pattern with the **Observer** pattern for real-time updates.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         VIEW LAYER                          │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │ MainView │──│OrderBook │  │  Trade   │  │  Chart   │   │
│  │          │  │   View   │  │ History  │  │   View   │   │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘   │
│       │             │  ▲           │             │          │
│       └─────────────┼──┼───────────┴─────────────┘          │
│                     │  │ Observer Pattern                   │
└─────────────────────┼──┼────────────────────────────────────┘
                      │  │
┌─────────────────────┼──┼────────────────────────────────────┐
│                     ▼  │      CONTROLLER LAYER              │
│  ┌──────────────────────────┐  ┌─────────────────────┐     │
│  │   TradingSimulator       │  │  SimulationSession  │     │
│  │  - Manages OrderBooks    │──│  - Event processing │     │
│  │  - Coordinates sessions  │  │  - Step execution   │     │
│  └──────────┬───────────────┘  └─────────────────────┘     │
└─────────────┼──────────────────────────────────────────────┘
              │
┌─────────────┼──────────────────────────────────────────────┐
│             ▼             MODEL LAYER                       │
│  ┌─────────────────────┐                                    │
│  │     OrderBook       │◄──Observer                         │
│  │  - Bids/Asks        │   Pattern                          │
│  │  - Price Levels     │                                    │
│  │  - Trade Matching   │                                    │
│  └──────────┬──────────┘                                    │
│             │                                                │
│    ┌────────┼────────┐                                      │
│    ▼        ▼        ▼                                      │
│  Order   Trade   Market Events                              │
│  Types   Data    & Assets                                   │
└─────────────────────────────────────────────────────────────┘
```

## Package Responsibilities

### Model Package
**Purpose**: Core business logic and data structures

**Key Components**:
- **TradableAsset** interface (Stock, Crypto implementations)
- **Order** hierarchy (LimitOrder, MarketOrder)
- **OrderBook**: Central matching engine with Observer support
- **PriceLevel**: Aggregates orders at specific price points
- **Trade**: Execution records
- **DTOs**: OrderBookSnapshot, PriceLevelInfo, OrderBookDepth, TopOfBook
- **OrderBookObserver** interface: For real-time updates
- **Market Events**: AddOrderEvent, CancelOrderEvent, ModifyOrderEvent
- **Data Export**: DataExporter interface with CsvDataExporter

**Key Responsibilities**:
- Order matching and execution
- Price-time priority enforcement
- Trade generation
- Order book state management
- Time-in-Force policy enforcement

### Controller Package
**Purpose**: Application logic and workflow coordination

**Key Components**:
- **TradingSimulator**: Main controller managing multiple OrderBooks and simulation sessions
- **SimulationSession**: Executes market events step-by-step for a specific OrderBook
- **EventLoader**: Loads market events from external sources
- Integration with DataExporter for results

**Key Responsibilities**:
- Simulation lifecycle management
- Event processing and orchestration
- Coordination between Model and View
- Session management
- Data import/export coordination

### View Package
**Purpose**: User interface and visualization

**Key Components**:
- **MainView**: UI coordinator and entry point
- **OrderBookView**: Live order book display (implements OrderBookObserver)
- **TradeHistoryView**: Trade execution display
- **ControlPanel**: User interaction controls
- **ChartView**: Price/volume visualization
- **StatusBar**: System status and notifications
- Supporting classes: PricePoint, VolumePoint, ChartType enum

**Key Responsibilities**:
- Real-time data display
- User input handling
- Chart rendering
- Simulation control interface
- Status updates and notifications

## Design Patterns

### 1. Observer Pattern
**Location**: Model ↔ View

**Implementation**:
- `OrderBookObserver` interface in Model
- `OrderBookView` implements this interface in View
- `OrderBook` notifies observers on state changes

**Benefits**:
- Decouples Model from View
- Enables real-time UI updates
- Supports multiple observers per OrderBook
- One-to-many dependency without tight coupling

### 2. Strategy Pattern
**Location**: Model (Order types)

**Implementation**:
- Abstract `Order` class
- Concrete implementations: `LimitOrder`, `MarketOrder`
- Different matching strategies based on order type

**Benefits**:
- Easy to add new order types
- Encapsulates order-specific behavior
- Simplifies OrderBook matching logic

### 3. Composite Pattern
**Location**: View (UI components)

**Implementation**:
- `MainView` composes multiple view components
- Each component is independently rendered
- Hierarchical UI structure

**Benefits**:
- Modular UI construction
- Component reusability
- Independent testing and maintenance

### 4. DTO Pattern
**Location**: Model (read-only data transfer)

**Implementation**:
- `OrderBookSnapshot`, `PriceLevelInfo`, `OrderBookDepth`, `TopOfBook`
- Immutable data structures for safe data sharing

**Benefits**:
- Safe data sharing between layers
- Prevents unintended state modification
- Clear API boundaries

## Data Flow

### Order Placement Flow
1. User places order via `ControlPanel`
2. `ControlPanel` calls `TradingSimulator.addOrder()`
3. `TradingSimulator` routes to appropriate `OrderBook`
4. `OrderBook` matches order and generates trades
5. `OrderBook` notifies observers (including `OrderBookView`)
6. `OrderBookView` updates display
7. `TradeHistoryView` shows new trades

### Simulation Flow
1. User loads events via `ControlPanel`
2. `EventLoader` reads events from file
3. `TradingSimulator` creates `SimulationSession`
4. User triggers step/run via `ControlPanel`
5. `SimulationSession` processes events
6. Events modify `OrderBook` state
7. Observers receive updates
8. View components refresh automatically

### Real-time Update Flow
```
OrderBook State Change
       │
       ├──> notify(snapshot)
       │
       ▼
OrderBookObserver.onOrderBookUpdate()
       │
       ├──> OrderBookView (implements observer)
       │
       ▼
   UI Update
```

## Technology Considerations

The design is **technology-agnostic** but supports:

### Desktop Application
- Java Swing/JavaFX
- Qt (C++)
- Electron (JavaScript/TypeScript)
- GTK (Python/C)

### Web Application
- React/Vue/Angular frontend
- RESTful API for Controller
- WebSocket for real-time updates

### Command-Line Interface
- Text-based UI with ncurses
- Simplified view components
- Same core architecture

## Performance Considerations

### Model Layer
- **Target**: < 20ms per trade execution
- **Optimization**: Efficient data structures (TreeMap for price levels)
- **Scalability**: Support 100 requests/second

### View Layer
- **Target**: < 500ms UI update latency
- **Optimization**: Throttled updates for high-frequency events
- **Selective rendering**: Update only changed components

### Observer Pattern Overhead
- Minimal: Only changed data sent via DTOs
- Batching: Multiple updates can be combined
- Async updates: Non-blocking notification

## Quality Attributes

Based on `ranking.md`:

1. **Functional Correctness**: Decimal precision, no floating-point errors
2. **Time Behavior**: 20ms trade processing, 500ms UI updates
3. **Scalability**: 100 requests/second capacity
4. **Learnability**: Intuitive UI for users with basic trading knowledge

## Extension Points

The architecture supports future extensions:

### Model Extensions
- New order types (Stop, Stop-Limit)
- Additional Time-in-Force policies
- Multiple asset types
- Advanced matching algorithms

### View Extensions
- Additional chart types
- Market depth visualization
- Portfolio management views
- Alert and notification systems
- Customizable dashboards

### Controller Extensions
- Live market data integration
- Multi-session management
- Advanced simulation scenarios
- Risk management features
- Backtesting capabilities

## Conclusion

This MVC architecture with Observer pattern provides:
- **Separation of Concerns**: Clear layer boundaries
- **Maintainability**: Independent component development
- **Testability**: Isolated testing of each layer
- **Scalability**: Performance-optimized design
- **Extensibility**: Easy to add new features
- **Technology Flexibility**: Core design independent of implementation technology
