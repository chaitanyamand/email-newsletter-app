FROM lukemathwalker/cargo-chef:latest-rust-slim as chef
WORKDIR /app
RUN apt update && apt install -y lld clang pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Planner Stage
FROM chef as planner
COPY . .
# Compute a lock-like file for our project
RUN cargo chef prepare --recipe-path recipe.json

# Builder Stage
FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
# Build our project dependencies, not application!
RUN cargo chef cook --release --recipe-path recipe.json
# Upto this point, if our dependencies are same, 
# all layers will be cached
COPY . .
ENV SQLX_OFFLINE=TRUE
# Build our project
RUN cargo build --release --bin emailnewsletter

# Runtime Stage
FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates libssl3 \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/emailnewsletter ./emailnewsletter
COPY configurations ./configurations
ENV APP_ENVIRONMENT=production
ENTRYPOINT ["./emailnewsletter"]
