FROM rust:1.88-alpine3.22 AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev

WORKDIR /app

# Copy manifest and source files
COPY . .

# Build the application
RUN cargo build --release

FROM alpine:3.22

# Install required runtime dependencies
RUN apk add --no-cache libgcc

# Set working directory
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/tee-worker-pre-compute .

# Run the application
ENTRYPOINT ["/app/tee-worker-pre-compute"]
