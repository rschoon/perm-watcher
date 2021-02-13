FROM rust:1

RUN apt-get update \
    && apt-get install -y jq \
    && rm -rf /var/lib/apt/lists/*
RUN rustup component add clippy
RUN cargo install cargo-deb
