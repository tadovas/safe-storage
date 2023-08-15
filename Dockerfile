FROM rust:1.71.1-slim as BUILDER
#RUN apt-get update && apt-get install -y pkg-config libssl-dev
WORKDIR /build
ADD . .
RUN cargo build -r --bin server

FROM debian:bookworm-slim
#RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=BUILDER /build/target/release/server server
ENTRYPOINT ["./server"]