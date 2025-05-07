#!/bin/sh

# See if dedoc can successfully fetch mock dock.json.

set -eu
. "$(dirname "$0")"/../scenario-utils.sh

# Ordinary fetch.
mock_dedoc fetch
mock_dedoc ls

# Mess with docs.json, ls should fail.
echo "lol" > "$DEDOC_HOME/docs.json"
! mock_dedoc ls

# Replace broken docs.json with a proper one.
mock_dedoc fetch -f
mock_dedoc ls

# Should fail due to recent docs.json.
! mock_dedoc fetch
