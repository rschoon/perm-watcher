
FROM buildpack-deps:stretch

ENV RUST_VERSION=1.65

RUN apt-get update \
    && apt-get install -y curl jq build-essential \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain $RUST_VERSION --profile minimal \
    && /root/.cargo/bin/rustup component add clippy
ENV PATH="/root/.cargo/bin:${PATH}"

# deb-s3 doesn't support control.tar.xz, so disable lzma feature
RUN cargo install --locked --no-default-features --version 1.36.0 cargo-deb
RUN cargo install --locked cargo-audit cargo-make
