# Multi-stage build for Merklith blockchain

# Stage 1: Builder
FROM rust:1.85-slim as builder

# Switch to nightly for edition2024 support
RUN rustup default nightly && \
    rustup component add rustfmt

# Install dependencies
RUN apt-get update && apt-get install -y \
    libclang-dev \
    llvm-dev \
    pkg-config \
    libssl-dev \
    cmake \
    g++ \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/merklith

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/
COPY sdk/ ./sdk/
COPY contracts/ ./contracts/
COPY benches/ ./benches/

# Build release binaries
RUN cargo build --release -p merklith-node -p merklith-cli

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r merklith && useradd -r -g merklith merklith

# Copy binaries from builder
COPY --from=builder /usr/src/merklith/target/release/merklith-node /usr/local/bin/
COPY --from=builder /usr/src/merklith/target/release/merklith /usr/local/bin/

# Create data directory
RUN mkdir -p /data && chown merklith:merklith /data

# Switch to non-root user
USER merklith

# Expose ports
# 8545 - RPC HTTP
# 8546 - RPC WebSocket
# 30303 - P2P
EXPOSE 8545 8546 30303

# Volume for data
VOLUME ["/data"]

# Default command
CMD ["merklith-node", "--data-dir", "/data"]
