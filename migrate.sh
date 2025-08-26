#!/bin/bash

# Migration management script for Bitrade

set -e

echo "🔄 Bitrade Migration Manager"

# Function to show usage
show_usage() {
    echo "Usage: $0 {dev|prod|status|reset}"
    echo ""
    echo "  dev   - Run migrations in development environment"
    echo "  prod  - Run migrations in production environment"
    echo "  status - Check migration status"
    echo "  reset  - Reset database (WARNING: This will delete all data!)"
    echo ""
}

# Check if we have a command
if [ $# -eq 0 ]; then
    show_usage
    exit 1
fi

case "$1" in
    dev)
        echo "📝 Running migrations in development environment..."
        ./run_migrations_dev.sh
        ;;
    prod)
        echo "🏭 Running migrations in production environment..."
        ./run_migrations.sh
        ;;
    status)
        echo "📊 Checking migration status..."
        if docker compose ps | grep -q postgres; then
            # Development environment
            docker compose -f docker-compose.dev.yml exec -T bitrade-engine-dev bash -c "
                cd /usr/src/bitrade/database && \
                DATABASE_URL='postgres://postgres:mysecretpassword@postgres:5432/postgres' \
                diesel migration list
            "
        else
            echo "❌ No running database found. Start the environment first."
            exit 1
        fi
        ;;
    reset)
        echo "⚠️  WARNING: This will delete all data in the database!"
        read -p "Are you sure you want to continue? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            echo "🗑️  Resetting database..."
            if docker compose ps | grep -q postgres; then
                # Development environment
                docker compose -f docker-compose.dev.yml exec -T bitrade-engine-dev bash -c "
                    cd /usr/src/bitrade/database && \
                    DATABASE_URL='postgres://postgres:mysecretpassword@postgres:5432/postgres' \
                    diesel database reset
                "
            else
                echo "❌ No running database found. Start the environment first."
                exit 1
            fi
        else
            echo "❌ Database reset cancelled."
            exit 1
        fi
        ;;
    *)
        echo "❌ Unknown command: $1"
        show_usage
        exit 1
        ;;
esac
