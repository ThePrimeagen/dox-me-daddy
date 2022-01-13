# syntax=docker/dockerfile:1
FROM rust:latest AS FETCH_THE_EFFIN_RUST
WORKDIR /app
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
COPY ./src/lib.rs ./src/lib.rs
RUN rustup default nightly
RUN cargo fetch
COPY ./src ./src
RUN cargo build --release --bin serve
RUN cargo install --path .

FROM debian:latest
EXPOSE 42069
ENV RUST_LOG=info
WORKDIR /app
RUN apt update && apt install -y ca-certificates
COPY --from=FETCH_THE_EFFIN_RUST /usr/local/cargo/bin/serve /app
CMD ["sh", "-c", "/app/serve"]


