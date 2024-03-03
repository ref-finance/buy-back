FROM rust:1.56.1

LABEL description="Container for builds"

RUN rustup default 1.56.1
RUN rustup target add wasm32-unknown-unknown

RUN apt-get update && apt-get install -y git less vim clang