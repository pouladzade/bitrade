#!/bin/bash

# Development migration script for Docker environment

echo "🔄 Running migrations in development environment..."

# Wait for PostgreSQL to be ready
echo "Waiting for PostgreSQL to be ready..."
until docker compose -f docker-compose.dev.yml exec -T postgres pg_isready -U postgres; do
  sleep 1
done

echo "PostgreSQL is ready!"

# Run migrations using Diesel CLI in the development container
echo "Running database migrations with Diesel..."
docker compose -f docker-compose.dev.yml exec -T bitrade-engine-dev bash -c "
    cd /usr/src/bitrade/database && \
    DATABASE_URL='postgres://postgres:mysecretpassword@postgres:5432/postgres' \
    diesel migration run
"

if [ $? -eq 0 ]; then
    echo "✅ Migrations completed successfully!"
else
    echo "❌ Migration failed!"
    exit 1
fi

echo "Database setup complete!"
