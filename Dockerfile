FROM rustlang/rust:nightly as builder

RUN USER=root cargo new --bin app
WORKDIR /usrc/src/app
COPY ./Cargo.toml ./Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/app/target \
    cargo build --release

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/app/target \
    cargo install --path .

FROM debian:bullseye-slim

RUN useradd -ms /bin/bash app

USER app
WORKDIR /app

COPY --from=builder /usr/local/cargo/bin/mindtrails /app/mindtrails