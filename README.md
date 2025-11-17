# Project: Multilevel Limit Orderbook

## Contributors
Keigo Healy, Tahasin Shadat

## Project Status

### Design Phase: ✅ COMPLETE

**View Package**: ✅ Complete and verified (see [VIEW-PACKAGE-COMPLETE.md](VIEW-PACKAGE-COMPLETE.md))

The complete system design following MVC architecture is available in `docs/design/design.puml`:
- **Model Package** (lines 3-248): Order book, trading logic, and data structures
- **Controller Package** (lines 250-292): Trading simulator and session management  
- **View Package** (lines 294-453): UI components with Observer pattern integration

### Documentation

- `docs/design/design.puml` - Complete UML class diagram (PlantUML)
- `docs/design/architecture-overview.md` - System architecture and patterns
- `docs/design/view-design-summary.md` - View package design details
- `docs/design/view-verification-report.md` - Quality verification report
- `docs/design/COMPLETION-SUMMARY.md` - View completion documentation
- `docs/design/view-todo-resolution.md` - TODO resolution summary
- `VIEW-PACKAGE-COMPLETE.md` - Quick status reference

## Dependencies
- Rust (planned implementation language)
- UI Framework (TBD - egui, iced, or gtk-rs recommended)
- PlantUML (for viewing design diagrams)

## Build Instructions

**Note**: This repository currently contains design documentation only. Implementation is pending.

### Viewing the Design

To view the PlantUML diagrams:

```bash
# Install PlantUML (requires Java)
# Option 1: Using package manager
sudo apt-get install plantuml  # Debian/Ubuntu
brew install plantuml          # macOS

# Option 2: Download JAR from plantuml.com

# Generate diagram
plantuml docs/design/design.puml

# This creates design.png in the same directory
```

### Next Steps for Implementation

1. Set up Rust project structure (`cargo init`)
2. Choose and add UI framework dependency
3. Implement Model package first
4. Implement Controller package
5. Implement View package (following design)
6. Wire up Observer pattern connections
7. Test against performance requirements (ranking.md)

## Design Highlights

- **Architecture**: Model-View-Controller (MVC)
- **Patterns**: Observer (Model→View updates), Strategy (order types), DTO (data transfer)
- **Components**: 9 View components, complete order book system, simulation framework
- **Requirements**: Designed for 20ms trade execution, 500ms GUI updates, 100 req/sec scalability

## Repository Structure

```
.
├── README.md                     # This file
├── VIEW-PACKAGE-COMPLETE.md      # View package status
├── ranking.md                    # Quality requirements
└── docs/design/
    ├── design.puml              # Complete UML design ⭐
    ├── architecture-overview.md # System architecture
    ├── view-design-summary.md   # View package details
    ├── view-verification-report.md  # Quality verification
    ├── view-todo-resolution.md  # TODO resolution
    └── COMPLETION-SUMMARY.md    # View completion docs
```

⭐ = Primary design document
