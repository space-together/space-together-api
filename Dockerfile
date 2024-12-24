# Build Stage
ARG RUST_VERSION=1.83.0
ARG APP_NAME=space-together-api

FROM rust:${RUST_VERSION}-slim-bookworm AS builder
# Set working directory
WORKDIR /code


# Install required packages
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy the manifest files and prefetch dependencies
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch --locked

# Copy the application source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime Stage
FROM bitnami/minideb:bookworm

# Install required runtime dependencies
RUN install_packages libssl-dev

# Set working directory
WORKDIR /app

# Add a non-root user
RUN useradd -ms /bin/bash appuser
USER appuser

# Copy the built binary from the builder stage
COPY --from=builder /code/target/release/space-together-api /app/space-together-api

# Expose application port
EXPOSE 2052

# Run the application
CMD ["/app/space-together-api"]
