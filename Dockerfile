# Build Stage
ARG RUST_VERSION=1.83.0
FROM rust:${RUST_VERSION}-slim-bookworm AS builder

WORKDIR /code

# Install required dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential pkg-config libssl-dev musl-tools && \
    rustup target add x86_64-unknown-linux-musl && \
    rm -rf /var/lib/apt/lists/*

# Install cargo-watch and cargo-chef
RUN cargo install cargo-watch cargo-chef

# Copy Cargo.toml and Cargo.lock files to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Generate dependency cache with cargo-chef
RUN cargo chef prepare --recipe-path recipe.json

# Fetch and cache dependencies
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release --target x86_64-unknown-linux-musl

# Development Stage for Live Reloading
FROM rust:${RUST_VERSION}-slim-bookworm AS dev

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /code/target/x86_64-unknown-linux-musl/release/space-together-api /app/space-together-api
COPY --from=builder /code/src /app/src
COPY --from=builder /code/Cargo.toml /app/Cargo.toml
COPY --from=builder /code/Cargo.lock /app/Cargo.lock

EXPOSE 2052

CMD ["cargo", "watch", "-q", "-c", "-w", "src/", "-x", "run"]

# Production Stage with Minimal Image
FROM scratch AS prod

COPY --from=builder /code/target/x86_64-unknown-linux-musl/release/space-together-api /space-together-api

ENTRYPOINT ["/space-together-api"]
