#!/bin/bash

# Wait for PostgreSQL to be ready
echo "Waiting for PostgreSQL to be ready..."
until docker compose exec -T postgres pg_isready -U postgres; do
  sleep 1
done

echo "PostgreSQL is ready!"

# Set up Diesel environment
echo "Setting up Diesel environment..."
export DATABASE_URL="postgres://postgres:mysecretpassword@localhost:5432/postgres"

# Run migrations using Diesel CLI
echo "Running database migrations with Diesel..."
cd database
diesel migration run

if [ $? -eq 0 ]; then
    echo "✅ Migrations completed successfully!"
else
    echo "❌ Migration failed!"
    exit 1
fi

echo "Database setup complete!"
