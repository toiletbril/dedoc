#!/bin/sh

I='dedoc-rust-cross'

# Build image if it's not present.
if ! docker image inspect $I > /dev/null 2>&1; then
  docker build --network=host -f Dockerfile -t $I `dirname $0`
fi

C='
for t in $RUSTTARGETS; do
  cargo build --release --target $t
done
'

# Compile the repo for targets defined in $RUSTTARGETS.
docker run --rm --network=host -v $PWD:/src $I sh -c "$C"
