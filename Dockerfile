# Image to cross-compile static rust binaries for targets specified in $TS.

FROM alpine:latest

SHELL ["sh", "-eu", "-c"]

RUN apk update
RUN apk add \
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
    ncurses \
    curl

ARG TS="x86_64-unknown-linux-musl x86_64-pc-windows-gnu"

# Install Rust and needed targets via Rustup, with the default toolchain set to
# nightly. llvm-components-preview is needed for code coverage.
RUN for t in $TS; do TC="${TC:-} -t $t"; done && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain nightly $TC \
       --component llvm-tools-preview && \
    git config --global --add safe.directory '*'

# Test whether we really installed Rust.
RUN stat "/root/.cargo" || exit 1

ENV PATH="/root/.cargo/bin:$PATH"

# Code coverage!
RUN cargo install grcov

ENV RUSTFLAGS="-C target-feature=+crt-static"
ENV RUSTTARGETS="$TS"

ENV RUST_BACKTRACE="1"

WORKDIR /src
