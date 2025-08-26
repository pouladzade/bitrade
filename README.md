# Bitrade Matching Engine

Bitrade is a high-performance, low-latency matching engine for spot trading built in Rust. It provides real-time order matching, trade execution, and market data services through gRPC APIs.

## Features

- ✅ **Order Matching**: Real-time limit and market order matching
- ✅ **Trade Execution**: Atomic trade execution with proper fee handling
- ✅ **Market Depth**: Real-time order book depth tracking
- ✅ **Multiple Markets**: Support for multiple trading pairs
- ✅ **gRPC APIs**: High-performance gRPC services for trading and querying
- ✅ **PostgreSQL**: Persistent storage with ACID transactions
- ✅ **Wallet Management**: Deposit, withdraw, and balance tracking
- ✅ **Market Statistics**: 24h high/low/volume tracking
- ✅ **Fee Treasury**: Automated fee collection and management
- ✅ **Order History**: Complete order and trade history
- ✅ **Order Cancellation**: Support for order cancellation and bulk operations

## Architecture

The project is organized as a Rust workspace with the following components:

- **`engine/`**: Core matching engine with gRPC trading API (port 50020)
- **`query/`**: Read-only gRPC query service (port 50021)
- **`database/`**: PostgreSQL schema, models, and repository layer
- **`common/`**: Shared utilities and types

## Quick Start

### Prerequisites

- Rust 1.89+
- PostgreSQL 15+
- Docker & Docker Compose (optional)

### Using Docker Compose (Recommended)

#### Production Mode

1. Clone the repository:

```bash
git clone <repository-url>
cd bitrade
```

2. Start the services:

```bash
./dev.sh prod
```

This will start:

- PostgreSQL database on port 5432
- Bitrade Engine on port 50020
- Bitrade Query Service on port 50021

#### Development Mode (with Hot Reloading)

1. Clone the repository:

```bash
git clone <repository-url>
cd bitrade
```

2. Start development environment with hot reloading:

```bash
./dev.sh dev
```

**Development Mode Features:**

- 🔥 **Hot Reloading**: Code changes automatically trigger rebuilds
- 📁 **Volume Mounting**: Live code changes without container rebuilds
- 🐛 **Debug Logging**: Detailed logs for development
- ⚡ **Fast Rebuilds**: Dependency caching for faster compilation
- 🔄 **Auto-restart**: Services restart automatically on code changes

### Manual Setup

1. Set up PostgreSQL:

```bash
# Create database
createdb postgres

# Run migrations
cd database
diesel migration run
```

2. Set environment variables:

```bash
cp env.example .env
# Edit .env with your database configuration
```

3. Build and run:

```bash
# Build the project
cargo build --release

# Run the matching engine
cargo run --bin bitrade

# Run the query service (in another terminal)
cargo run --bin query
```

## Database Management

### Migrations

Bitrade uses Diesel CLI for database migrations. The following commands are available:

```bash
# Run migrations in development environment
./migrate.sh dev

# Run migrations in production environment
./migrate.sh prod

# Check migration status
./migrate.sh status

# Reset database (WARNING: Deletes all data!)
./migrate.sh reset
```

### Creating New Migrations

To create a new migration:

```bash
cd database
diesel migration generate migration_name
```

This will create a new migration file in `database/migrations/`.

## API Documentation

### Trading Engine API (Port 50020)

The trading engine provides the following gRPC services:

#### Market Management

- `CreateMarket`: Create a new trading pair
- `StartMarket`: Start accepting orders for a market
- `StopMarket`: Stop accepting orders for a market

#### Order Management

- `AddOrder`: Place a new order (limit or market)
- `CancelOrder`: Cancel a specific order
- `CancelAllOrders`: Cancel all orders for a market

#### Wallet Operations

- `Deposit`: Deposit funds to a user's wallet
- `Withdraw`: Withdraw funds from a user's wallet
- `GetBalance`: Get current balance for a user/asset

### Query Service API (Port 50021)

The query service provides read-only access to:

#### Market Data

- `GetMarket`: Get market information
- `ListMarkets`: List all available markets
- `GetMarketStats`: Get 24h market statistics

#### Order Data

- `GetOrder`: Get specific order details
- `ListOrders`: List orders with filtering and pagination

#### Trade Data

- `ListTrades`: List trades with filtering and pagination
- `GetUserTrades`: Get trades for a specific user

#### Wallet Data

- `GetWallet`: Get wallet balance for a user/asset
- `ListWallets`: List wallets with filtering and pagination

#### Fee Treasury

- `GetFeeTreasury`: Get fee treasury information

## Configuration

The application can be configured through environment variables:

| Variable                     | Default                                                   | Description                   |
| ---------------------------- | --------------------------------------------------------- | ----------------------------- |
| `DATABASE_URL`               | `postgres://postgres:mysecretpassword@localhost/postgres` | PostgreSQL connection string  |
| `SERVER_HOST`                | `[::]`                                                    | Server host address           |
| `SERVER_PORT`                | `50020`                                                   | Server port                   |
| `RUST_LOG`                   | `info`                                                    | Logging level                 |
| `BITRADE_DATABASE_POOL_SIZE` | `10`                                                      | Database connection pool size |

## Development

### Running Tests

```bash
cargo test
```

### Code Formatting

```bash
cargo fmt
```

### Database Migrations

```bash
cd database
diesel migration generate <migration_name>
diesel migration run
diesel migration revert
```

### Building for Production

```bash
cargo build --release
```

## Project Structure

```
bitrade/
├── engine/                 # Core matching engine
│   ├── src/
│   │   ├── main.rs        # Application entry point
│   │   ├── grpc/          # gRPC service definitions
│   │   ├── market/        # Market management
│   │   ├── order_book/    # Order book and matching logic
│   │   ├── models/        # Data models
│   │   ├── validation/    # Input validation
│   │   └── wallet/        # Wallet operations
│   └── Cargo.toml
├── query/                  # Query service
│   ├── src/
│   │   ├── main.rs        # Query service entry point
│   │   ├── server.rs      # gRPC server setup
│   │   ├── service.rs     # Query service implementation
│   │   └── adapter.rs     # Data adapters
│   └── Cargo.toml
├── database/              # Database layer
│   ├── src/
│   │   ├── models/        # Database models
│   │   ├── repository/    # Data access layer
│   │   ├── provider/      # Database provider traits
│   │   └── filters/       # Query filters
│   ├── migrations/        # Database migrations
│   └── Cargo.toml
├── common/                # Shared utilities
│   ├── src/
│   │   ├── utils/         # Common utilities
│   │   └── db/           # Database utilities
│   └── Cargo.toml
├── docker-compose.yml     # Development environment
├── Dockerfile            # Production container
└── README.md
```

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## License

This project is licensed under the Apache 2.0 License - see the [LICENSE](LICENSE) file for details.

## Performance

The matching engine is designed for high-performance trading:

- **Latency**: Sub-millisecond order matching
- **Throughput**: 100,000+ orders per second
- **Concurrency**: Multi-threaded order processing
- **Persistence**: ACID-compliant trade execution

## Security

- Input validation on all API endpoints
- SQL injection protection through Diesel ORM
- Proper error handling without information leakage
- Non-root container execution

## Monitoring

The application provides comprehensive logging and can be monitored through:

- Structured logging with configurable levels
- gRPC health checks
- Database connection monitoring
- Order book depth tracking
