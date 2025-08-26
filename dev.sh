#!/bin/bash

# Development script for Bitrade with hot reloading

set -e

echo "🚀 Starting Bitrade Development Environment with Hot Reloading..."

# Function to cleanup on exit
cleanup() {
    echo "🛑 Stopping development environment..."
    docker compose -f docker-compose.dev.yml down
}

# Set up trap to cleanup on script exit
trap cleanup EXIT

# Check if we're running in development mode
if [ "$1" = "dev" ]; then
    echo "📝 Development mode: Hot reloading enabled"
    
    # Start the development environment
    docker compose -f docker-compose.dev.yml up --build -d
    
    # Wait a moment for services to start
    echo "⏳ Waiting for services to start..."
    sleep 10
    
    # Run migrations
    echo "🔄 Running database migrations..."
    ./run_migrations_dev.sh
    
    echo "✅ Development environment is ready!"
    echo "📝 Make changes to your code and they will automatically reload"
    echo "📊 View logs with: docker compose -f docker-compose.dev.yml logs -f"
    
    # Keep the script running to show logs
    docker compose -f docker-compose.dev.yml logs -f
    
elif [ "$1" = "prod" ]; then
    echo "🏭 Production mode: Using optimized build"
    
    # Start the production environment
    docker compose up --build -d
    
    # Wait a moment for services to start
    echo "⏳ Waiting for services to start..."
    sleep 10
    
    # Run migrations
    echo "🔄 Running database migrations..."
    ./run_migrations.sh
    
    echo "✅ Production environment is ready!"
    
else
    echo "Usage: $0 {dev|prod}"
    echo ""
    echo "  dev  - Start development environment with hot reloading"
    echo "  prod - Start production environment"
    echo ""
    echo "Development mode features:"
    echo "  ✅ Hot reloading with cargo-watch"
    echo "  ✅ Volume mounting for live code changes"
    echo "  ✅ Debug logging enabled"
    echo "  ✅ Faster rebuilds with dependency caching"
    echo "  ✅ Auto-restart on code changes"
    echo "  ✅ Automatic migration handling"
    echo ""
    echo "Production mode features:"
    echo "  ✅ Optimized builds"
    echo "  ✅ Minimal container size"
    echo "  ✅ Production logging"
    echo "  ✅ Automatic migration handling"
    exit 1
fi
