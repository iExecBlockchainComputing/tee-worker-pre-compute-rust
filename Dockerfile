# Multi-stage Dockerfile for iExec tee-worker-pre-compute API
# Stage 1: Build stage with Rust toolchain
FROM rust:1.88-alpine3.20 AS builder

# Install build dependencies
RUN apk add --no-cache \
    openssl-dev \
    musl-dev \
    gcc \
    libc-dev

WORKDIR /app

# Copy Cargo files first for better caching
COPY Cargo.* /app/

# Create a dummy main.rs to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source code
COPY src/ /app/src/

# Build the application
RUN cargo build --release --bin tee-worker-pre-compute

# Stage 2: Runtime stage with minimal image
FROM alpine:3.22.1 AS runtime

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    openssl \
    libc6-compat

# Create non-root user for security
RUN addgroup -g 1001 -S appgroup && \
    adduser -u 1001 -S appuser -G appgroup

# Set working directory
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/tee-worker-pre-compute /app/tee-worker-pre-compute

# Copy any necessary runtime files
COPY --from=builder /app/tests_resources/ ./tests_resources/ 2>/dev/null || true

# Change ownership to non-root user
RUN chown -R appuser:appgroup /app

# Switch to non-root user
USER appuser

# Set environment variables
ENV RUST_LOG=info

# Run the application
CMD ["/app/tee-worker-pre-compute"]
