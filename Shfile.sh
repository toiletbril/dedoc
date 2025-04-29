#!/bin/sh

# A splendid alternative to Makefile, used for cross-compiling and running
# integration tests.

set -eu

IMG='dedoc-rust-cross'

BUILD_CMD='
for T in $RUSTTARGETS; do
  cargo build --profile "$BUILDMODE" --target "$T" --target-dir target-docker
done
'
TEST_CMD='
cargo build --target x86_64-unknown-linux-musl --target-dir target-docker &&
./test/run-tests.sh
'

C="${1:-}"

case $C in
"make-image")
  if docker image inspect "$IMG" > /dev/null 2>&1; then
    docker rmi -f "$IMG"
    DOCKER_TARGET="$(dirname "$0")/target-docker"
    if test -d "$DOCKER_TARGET"; then
      sudo rm -r "$DOCKER_TARGET"
    fi
  fi
  docker build --network=host -f Dockerfile -t "$IMG" "$(dirname "$0")"
  ;;
"cross-compile")
  docker run --pull=never --rm --network=host -e BUILDMODE="release" -v \
             "$PWD":/src $IMG sh -c "$BUILD_CMD"
  ;;
"test")
  docker run --pull=never --rm --network=host -e BUILDMODE="dev" -v \
             "$PWD":/src $IMG sh -c "$TEST_CMD"
  ;;
*)
  echo "USAGE: $0 <make-image/cross-compile/test>"
  exit 1
  ;;
esac
