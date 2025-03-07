#!/bin/sh

set -eu

IMG='dedoc-rust-cross'
BLD_CMD='
for t in $RUSTTARGETS; do
  cargo build --profile $BUILDMODE --target $t
done
'

C="${1:-}"

case $C in
"build")
  docker run --rm --network=host -e BUILDMODE="debug" -v $PWD:/src $IMG \
         sh -c "$BLD_CMD"
  ;;
"test")
  docker run --rm --network=host -e BUILDMODE="release" -v $PWD:/src $IMG \
         sh -c "$BLD_CMD && test/run_tests.sh"
  ;;
*)
  echo "USAGE: $0 <build/test>"
  exit 1
  ;;
esac
