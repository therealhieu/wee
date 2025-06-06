# --- Cache dependencies ---
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY .. .
RUN cargo chef prepare --recipe-path recipe.json

# --- Build the application ---
FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
COPY .. .
RUN cargo build --release -p wee-shorten

# --- Run the application ---
FROM debian:bookworm-slim
WORKDIR /app

COPY --from=builder /app/target/release/wee-shorten /app/wee-shorten

EXPOSE 4000
CMD ["./wee-shorten"]

