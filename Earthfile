
VERSION 0.7

setup:
    FROM docker.io/buildpack-deps:stretch

    ENV RUST_VERSION=1.75

    RUN echo 'deb http://archive.debian.org/debian/ stretch contrib main non-free' > /etc/apt/sources.list

    RUN apt-get update \
        && apt-get install -y curl jq build-essential \
        && rm -rf /var/lib/apt/lists/*

    RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
        sh -s -- -y --default-toolchain $RUST_VERSION --profile minimal \
        && /root/.cargo/bin/rustup component add clippy
    ENV PATH="/root/.cargo/bin:${PATH}"
    ENV CARGO_INCREMENTAL=0

    # deb-s3 doesn't support control.tar.xz, so disable lzma feature
    RUN cargo install --locked --no-default-features --version 1.36.0 cargo-deb
    RUN cargo install --locked cargo-audit cargo-make

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
    ARG CI_COMMIT_TAG
    RUN cargo build --release --color always
    RUN CI_COMMIT_TAG=$CI_COMMIT_TAG DISTDIR=/dist ci/package.sh
    SAVE ARTIFACT /dist
