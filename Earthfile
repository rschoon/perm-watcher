
VERSION 0.7

setup:
    ARG rust_version=1.76
    FROM docker.io/buildpack-deps:stretch

    RUN echo 'deb http://archive.debian.org/debian/ stretch contrib main non-free' > /etc/apt/sources.list

    RUN apt-get update \
        && apt-get install -y curl jq build-essential \
        && rm -rf /var/lib/apt/lists/*

    RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
        sh -s -- -y --default-toolchain $rust_version --profile minimal \
        && /root/.cargo/bin/rustup component add clippy
    ENV PATH="/root/.cargo/bin:${PATH}"
    ENV CARGO_INCREMENTAL=0

    RUN cargo install --locked cargo-audit cargo-make cargo-deb

fetch:
    FROM +setup
    RUN mkdir /build
    WORKDIR /build
    COPY --dir Makefile.toml Cargo.toml Cargo.lock .
    RUN mkdir src && touch src/main.rs
    RUN cargo make fetch
    COPY --dir . .

audit:
    FROM +fetch
    RUN --no-cache cargo make audit

test:
    FROM +fetch
    RUN cargo test --all

clippy:
    FROM +fetch
    RUN cargo clippy

build:
    FROM +fetch
    ARG REF_TYPE
    RUN cargo build --release --color always
    RUN REF_TYPE=$REF_TYPE DISTDIR=/dist ci/package.sh
    SAVE ARTIFACT /dist
