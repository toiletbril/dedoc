#!/bin/sh

I='dedoc-rust-cross'

# Build image if it's not present.
if ! docker image inspect $I > /dev/null 2>&1; then
  docker build --network=host -f Dockerfile -t $I `dirname $0`
fi

C='
TS="x86_64-unknown-linux-musl x86_64-pc-windows-gnu"

for t in $TS; do
  cargo build --release --target $t
done
'

# Compile the repo for x86_64 Linux and Windows.
docker run --rm --network=host -v $PWD:/src $I sh -c "$C"
