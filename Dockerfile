# Image to cross-compile static rust binaries for x86_64-pc-windows-gnu and
# x86_64-unknown-linux-musl.

FROM alpine:latest

ARG H="/root"

RUN apk update
RUN apk add --no-cache \
    build-base \
    musl-dev \
    linux-headers \
    mingw-w64-gcc \
    curl

# Install Rust and needed targets via Rustup, with the default toolchain set to
# nightly.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y \
       --default-toolchain nightly \
       -t x86_64-unknown-linux-musl \
       -t x86_64-pc-windows-gnu

ENV PATH="$H/.cargo/bin:$PATH"

# Make sure rustc knows we're static.
ENV RUSTFLAGS="-C target-feature=+crt-static"

WORKDIR /src
