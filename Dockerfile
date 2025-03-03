# Build stage
FROM rust:1.84-buster as builder
WORKDIR /app

# accept the build argument
ARG DATABASE_URL

ENV DATABASE_URL=$DATABASE_URL

COPY . .

RUN cargo build --release

WORKDIR /usr/local/bin

COPY --from=builder /app/target/release/space-together-api .

CMD [ "./space-together-api" ]