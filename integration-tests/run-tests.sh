#!/bin/sh

# Entrypoint for running integration tests inside a Docker image. See README.md
# and top-level Shfile.sh for more information.

set -e

# Make sure we're in the right directory.
REAL_PWD="$(realpath "$PWD")"
ACTUAL_PWD=$(realpath "$(dirname "$0")")

if ! test "$REAL_PWD" = "$ACTUAL_PWD"; then
  echo "Changing directory to $ACTUAL_PWD.." >&2
  cd "$ACTUAL_PWD" && exec ./"$(basename "$0")"
fi

. "$(dirname "$0")"/scenario-utils.sh

# Make sure we're in docker container to avoid ruining real enviroment.
if test -z "$RUSTTARGETS"; then
  log_err_and_die "Please run this script from under provided docker image."
fi

set -eu

run_test() {
if test -z "$1"; then
  log_err_and_die "No argument given to run_test()."
elif ! test -e "$1"; then
  log_err_and_die "Non-existent scenario '$1' given to run_test()." >&2
else
  log "============ Running $1... ============"
  $1
  log "============ $1 completed sucessfully. ============"
fi
}

# Cleanup on exit.
trap 'kill -9 $(cat "$MOCK_SERVER_PID_PATH") && \
      rm "$MOCK_SERVER_PID_PATH" "$KEY_PATH" "$CERT_PATH"' \
     EXIT

# Prepare the environment.
mkdir -p "$DEDOC_HOME"
override_host devdocs.io 127.0.0.1
override_host documents.devdocs.io 127.0.0.1
make_sure_mock_cert_is_installed

(start_mock_file_server)

# Use every file in scenario/ as a test.
for F in ./scenario/*.sh; do
  run_test "$F"
done

# Collect coverage!
grcov --binary-path "../target-docker/x86_64-unknown-linux-musl/debug" \
      --branch -s ../src -t html -o "$COVERAGE_DIR/report" \
      "$COVERAGE_DIR/profraw/" 

log "👍"
