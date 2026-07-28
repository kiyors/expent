FROM rust:1.91.1-slim AS chef
WORKDIR /app
# Install dependencies needed for compiling
RUN apt-get update && apt-get install -y perl make pkg-config libssl-dev gcc && rm -rf /var/lib/apt/lists/*
# Install cargo-chef
RUN cargo install cargo-chef --locked

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release --bin server && \
    cp target/release/server /tmp/server

FROM debian:bookworm-slim AS runner
WORKDIR /app

# Install runtime dependencies like openssl and ca-certificates
RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the cached tmp location
COPY --from=builder /tmp/server /usr/local/bin/

# Make sure we run with info logs by default
ENV RUST_LOG=info
ENV PORT=7878

EXPOSE 7878

CMD ["server"]
