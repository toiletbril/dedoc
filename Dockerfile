# Image to cross-compile static rust binaries for targets specified in $TS.

FROM alpine:latest

ARG H="/root"
ARG TS="x86_64-unknown-linux-musl x86_64-pc-windows-gnu"

RUN apk update
RUN apk add --no-cache \
    build-base \
    musl-dev \
    linux-headers \
    mingw-w64-gcc \
    git \
    openssl \
    openssl-libs-static \
    ca-certificates \
    pkgconf \
    python3 \
    curl

# Install Rust and needed targets via Rustup, with the default toolchain set to
# nightly.
RUN for t in $TS; do TC="$TC -t $t"; done && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain nightly $TC && \
    git config --global --add safe.directory '*'

ENV PATH="$H/.cargo/bin:$PATH"

ENV RUSTFLAGS="-C target-feature=+crt-static"
ENV RUSTTARGETS="$TS"

ENV RUST_BACKTRACE="1"

WORKDIR /src
