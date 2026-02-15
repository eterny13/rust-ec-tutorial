#!/bin/bash
set -e

echo "🚀 Starting Integration Test Setup..."

# 1. Start Docker Infrastructure
echo "📦 Starting Docker services..."
cd etc/docker
docker compose up -d
echo "⏳ Waiting for DB and Kafka..."
sleep 20 # Wait for MySQL and Kafka to be ready

echo "🔧 Creating Kafka Topics..."
docker compose exec -T kafka kafka-topics --create --if-not-exists --bootstrap-server localhost:9092 --topic order-events --partitions 1 --replication-factor 1 || true
docker compose exec -T kafka kafka-topics --create --if-not-exists --bootstrap-server localhost:9092 --topic payment-events --partitions 1 --replication-factor 1 || true
docker compose exec -T kafka kafka-topics --create --if-not-exists --bootstrap-server localhost:9092 --topic inventory-events --partitions 1 --replication-factor 1 || true
cd ../..

# 2. Run Migrations
echo "🔄 Running Migrations..."
# Install sqlx-cli if not present? Assumed present or we use local cargo run if possible?
# Better to assume user has sqlx-cli or use a temporary way. 
# For this environment, let's assume sqlx is available or we can just try to run it.
# If not, we might fail.
# Alternatively, use mysql client to source migration files? No, sqlx migrate is better.

export DATABASE_URL="mysql://ecuser:ecpassword@localhost:3306/order_db"
if [ -d "order/db/migrations" ]; then
    sqlx migrate run --source order/db/migrations
fi

export DATABASE_URL="mysql://ecuser:ecpassword@localhost:3306/inventory_db"
if [ -d "inventory/db/migrations" ]; then
    sqlx migrate run --source inventory/db/migrations
fi

export DATABASE_URL="mysql://ecuser:ecpassword@localhost:3306/payment_db"
if [ -d "payment/db/migrations" ]; then
    sqlx migrate run --source payment/db/migrations
fi

# 3. Seed Data
echo "🌱 Seeding Data..."
# Prod ID: prod_456
docker compose -f etc/docker/docker-compose.yml exec -T mysql mysql -uecuser -pecpassword inventory_db -e "INSERT INTO inventories (id, available_quantity, reserved_quantity, created_at, updated_at) VALUES ('prod_456', 10, 0, NOW(), NOW()) ON DUPLICATE KEY UPDATE available_quantity = 10;"

# 4. Start Services
echo "🎬 Starting Services..."
mkdir -p logs
export RUST_LOG=info

export DATABASE_URL="mysql://ecuser:ecpassword@localhost:3306/order_db"
export KAFKA_BROKERS="localhost:9092"
cargo run --bin order > logs/order.log 2>&1 &
ORDER_PID=$!
echo "Order Service PID: $ORDER_PID"

export DATABASE_URL="mysql://ecuser:ecpassword@localhost:3306/payment_db"
cargo run --bin payment > logs/payment.log 2>&1 &
PAYMENT_PID=$!
echo "Payment Service PID: $PAYMENT_PID"

export DATABASE_URL="mysql://ecuser:ecpassword@localhost:3306/inventory_db"
cargo run --bin inventory > logs/inventory.log 2>&1 &
INVENTORY_PID=$!
echo "Inventory Service PID: $INVENTORY_PID"

echo "⏳ Waiting for services to startup..."
sleep 10

# 5. Run Integration Tests
echo "🧪 Running Integration Tests..."
export RUST_LOG=info
if cargo run -p integration-tests; then
    echo "✅ Tests Passed!"
    RESULT=0
else
    echo "❌ Tests Failed!"
    RESULT=1
fi

# 6. Cleanup
echo "🧹 Cleaning up..."
kill $ORDER_PID $PAYMENT_PID $INVENTORY_PID || true
# Optional: docker compose down

exit $RESULT
