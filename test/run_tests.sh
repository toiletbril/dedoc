#!/bin/sh

# Make sure we're in docker container to avoid ruining real enviroment.
if test -z "$RUSTTARGETS"; then
  echo "Please run this script from under provided docker image." >&2
  exit 1
fi

set -eu

# Make sure we're in test/ directory.
ACTUAL_PWD=`realpath $(dirname "$0")`
REAL_PWD=`realpath "$PWD"`

if ! test "$REAL_PWD" = "$ACTUAL_PWD"; then
  echo "Changing directory to $ACTUAL_PWD.." >&2
  cd "$ACTUAL_PWD" && exec ./`basename "$0"`
fi

run_test() {
if test -z "$1"; then
  echo "No argument given to run_test()." >&2
  return
elif ! test -e "$1"; then
  echo "Non-existent scenario '$1' given to run_test()." >&2
  return
fi

echo "Running $1.." >&2

$1
}

# Use every file in scenario/ as a test.
for F in ./scenario/*.sh; do
  run_test "$F"
done
