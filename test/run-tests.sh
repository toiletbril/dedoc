#!/bin/sh

# Entrypoint for running integration tests inside a Docker image. See README.md
# for more information.

. "$(dirname "$0")"/scenario-utils.sh

# Make sure we're in docker container to avoid ruining real enviroment.
if test -z "$RUSTTARGETS"; then
  log_err_and_die "Please run this script from under provided docker image."
fi

set -eu

# Make sure we're in test/ directory.
ACTUAL_PWD=$(realpath "$(dirname "$0")")
REAL_PWD="$(realpath "$PWD")"

if ! test "$REAL_PWD" = "$ACTUAL_PWD"; then
  log "Changing directory to $ACTUAL_PWD.."
  cd "$ACTUAL_PWD" && exec ./"$(basename "$0")"
fi

run_test() {
if test -z "$1"; then
  log_err_and_die "No argument given to run_test()."
elif ! test -e "$1"; then
  log_err_and_die "Non-existent scenario '$1' given to run_test()." >&2
else
  log "Running $1.."
  $1
  log "$1 completed sucessfully."
fi
}

# Prepare the environment.
run_test ./setup.sh

# Use every file in scenario/ as a test.
for F in ./scenario/*.sh; do
  run_test "$F"
done
