# Use the official Rust image as a base
FROM rust:1.89-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Install diesel CLI for migrations
RUN cargo install diesel_cli --no-default-features --features postgres

# Set working directory
WORKDIR /usr/src/bitrade

# Copy the entire project
COPY . .

# Build both applications
RUN cargo build --release --bin bitrade --bin spot-query

# Create a new stage with a minimal image
FROM debian:bookworm-slim

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -r -s /bin/false bitrade

# Set working directory
WORKDIR /app

# Copy the binaries from builder stage
COPY --from=builder /usr/src/bitrade/target/release/bitrade /app/bitrade
COPY --from=builder /usr/src/bitrade/target/release/spot-query /app/query

# Change ownership to the bitrade user
RUN chown -R bitrade:bitrade /app

# Switch to non-root user
USER bitrade

# Expose ports
EXPOSE 50020 50021

# Set environment variables
ENV RUST_LOG=info

# Default command
CMD ["./bitrade"]
