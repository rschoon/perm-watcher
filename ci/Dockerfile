
FROM buildpack-deps:stretch

ENV RUST_VERSION=1.53

RUN apt-get update \
    && apt-get install -y curl jq build-essential \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain $RUST_VERSION --profile minimal \
    && /root/.cargo/bin/rustup component add clippy
ENV PATH="/root/.cargo/bin:${PATH}"

# deb-s3 doesn't support control.tar.xz, so disable lzma feature
RUN cargo install cargo-deb --no-default-features
RUN cargo install cargo-audit
