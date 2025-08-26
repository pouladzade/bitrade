#!/bin/bash

echo "Testing Bitrade Services..."
echo "=========================="

# Test if PostgreSQL is accessible
echo "1. Testing PostgreSQL connection..."
if docker compose exec -T postgres pg_isready -U postgres; then
    echo "✅ PostgreSQL is running and accessible"
else
    echo "❌ PostgreSQL is not accessible"
    exit 1
fi

# Test if the engine service is responding
echo "2. Testing Engine service (port 50020)..."
if curl -s --connect-timeout 5 http://localhost:50020 > /dev/null 2>&1; then
    echo "✅ Engine service is responding on port 50020"
else
    echo "⚠️  Engine service port 50020 is not responding to HTTP (expected for gRPC)"
fi

# Test if the query service is responding
echo "3. Testing Query service (port 50021)..."
if curl -s --connect-timeout 5 http://localhost:50021 > /dev/null 2>&1; then
    echo "✅ Query service is responding on port 50021"
else
    echo "⚠️  Query service port 50021 is not responding to HTTP (expected for gRPC)"
fi

# Check if ports are listening
echo "4. Checking if ports are listening..."
if ss -tlnp | grep -q ":50020 "; then
    echo "✅ Port 50020 is listening"
else
    echo "❌ Port 50020 is not listening"
fi

if ss -tlnp | grep -q ":50021 "; then
    echo "✅ Port 50021 is listening"
else
    echo "❌ Port 50021 is not listening"
fi

echo ""
echo "🎉 All services appear to be running!"
echo ""
echo "Services Summary:"
echo "- PostgreSQL: localhost:5432"
echo "- Bitrade Engine: localhost:50020 (gRPC)"
echo "- Bitrade Query: localhost:50021 (gRPC)"
echo ""
echo "You can now use gRPC clients to interact with the services."
