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
./integration-tests/run-tests.sh
'

clean_docker_target() {
DOCKER_TARGET="$(dirname "$0")/target-docker"
if test -d "$DOCKER_TARGET"; then
  rm -r "$DOCKER_TARGET"
fi
}

remove_docker_image() {
if docker image inspect "$IMG" > /dev/null 2>&1; then
  docker rmi -f "$IMG"
fi
}

C="${1:-}"

case $C in
"make-image")
  remove_docker_image
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
"clean")
  cargo clean
  remove_docker_image
  clean_docker_target
  ;;
*)
  echo "USAGE: $0 <make-image/cross-compile/test/clean>"
  exit 1
  ;;
esac
