# Builder Stage
FROM rust:slim AS builder
WORKDIR /app
RUN apt update && \
    apt install -y lld clang pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*
COPY . .
ENV SQLX_OFFLINE=TRUE
RUN cargo build --release

# Runtime Stage
FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update && \
    apt-get install -y --no-install-recommends openssl ca-certificates && \
    rm -rf /var/lib/apt/lists/*
RUN apt-get update && apt-get install -y libssl3
COPY --from=builder /app/target/release/emailnewsletter ./emailnewsletter
COPY configurations ./configurations
ENV APP_ENVIRONMENT=production
ENTRYPOINT ["./emailnewsletter"]
