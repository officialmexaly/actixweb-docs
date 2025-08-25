# Build stage
FROM rust:1.76-slim AS builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml ./
COPY entity/Cargo.toml ./entity/
COPY migration/Cargo.toml ./migration/

# Create dummy source files to cache dependencies
RUN mkdir -p src entity/src migration/src && \
    echo "fn main() {}" > src/main.rs && \
    echo "pub fn dummy() {}" > entity/src/lib.rs && \
    echo "pub fn dummy() {}" > migration/src/lib.rs

# Build dependencies (this will create a new Cargo.lock)
RUN cargo build --release && rm -rf src entity/src migration/src target/release/deps/mexalydocs*

# Copy actual source code
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage  
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false appuser

WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/mexalydocs .

# Change ownership
RUN chown -R appuser:appuser /app
USER appuser

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the application
CMD ["./mexalydocs"]