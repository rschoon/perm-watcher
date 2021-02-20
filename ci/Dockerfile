
FROM buildpack-deps:stretch

ENV RUST_VERSION=1.49

RUN apt-get update \
    && apt-get install -y curl jq build-essential \
    && rm -rf /var/lib/apt/lists/*
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain $RUST_VERSION \
    && (cd /usr/local/bin; for i in $HOME/.cargo/bin/*; do ln -s $i; done)
RUN rustup component add clippy
RUN cargo install cargo-deb && ln -s $HOME/.cargo/bin/cargo-deb /usr/local/bin
