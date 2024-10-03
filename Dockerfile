FROM rust:latest

WORKDIR /app

RUN apt update && apt install lld clang -y

COPY . .

ENV SQLX_OFFLINE TRUE

RUN cargo build --release

ENTRYPOINT [ "./target/debug/emailnewsletter" ]

