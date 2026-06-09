# ============================================
# Stage 1: Build the Rust binary
# ============================================
FROM rust:1.83-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main to cache dependency compilation
RUN mkdir src && echo "fn main() {}" > src/main.rs \
    && cargo build --release \
    && rm -rf src

# Copy actual source code
COPY src ./src
COPY docs/migrations ./docs/migrations

# Build the real binary (touch main.rs to invalidate cache for our code)
RUN touch src/main.rs && cargo build --release

# ============================================
# Stage 2: Minimal runtime image
# ============================================
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/trend-monitor /app/trend-monitor

# Copy config and migrations
COPY config.toml /app/config.toml
COPY docs/migrations /app/docs/migrations

# Create data directory for SQLite
RUN mkdir -p /app/docs/data

# Expose the application port
EXPOSE 3000

# Volume for persistent SQLite data
VOLUME ["/app/docs/data"]

# Run all modules + API server (no mode argument needed)
ENTRYPOINT ["./trend-monitor"]
CMD ["config.toml"]
