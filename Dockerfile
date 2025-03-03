# Stage 1: Build the Rust application
FROM rust:1.84 as builder

# Set working directory inside the container
WORKDIR /usr/src/app

# Copy Cargo files separately to leverage Docker cache
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Install dependencies and build the application in release mode
RUN cargo build --release

# Stage 2: Create a lightweight image to run the application
FROM debian:bookworm-slim

# Install only necessary dependencies
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Set working directory inside the container
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/space-together-api .

# Expose the application's port (adjust as needed)
EXPOSE 20045

# Set the default command to run the application
CMD ["./space-together-api"]
