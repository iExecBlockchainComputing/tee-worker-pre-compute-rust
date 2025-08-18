FROM rust:1.88 as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application in release mode
RUN cargo build --release

# Runtime stage - use a minimal base image
FROM alpine:3.22.1 AS runtime

# Install runtime dependencies if needed
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder stage
COPY --from=builder /app/target/release/tee-worker-pre-compute /usr/local/bin/tee-worker-pre-compute

ENTRYPOINT ["tee-worker-pre-compute"]
