#!/bin/sh

# See if dedoc can successfully fetch mock dock.json.

set -eu
. "$(dirname "$0")"/../scenario-utils.sh

# Ordinary fetch.
wrapped_dedoc fetch
wrapped_dedoc ls

# Mess with docs.json, ls should fail.
echo "lol" > "$DEDOC_HOME/docs.json"
! wrapped_dedoc ls

# Replace broken docs.json with a proper one.
wrapped_dedoc fetch -f
wrapped_dedoc ls

# Should fail due to recent docs.json.
! wrapped_dedoc fetch

# Test with a different $DEDOC_HOME.
export DEDOC_HOME="/root/.dedoc2"
! wrapped_dedoc ls
! wrapped_dedoc ft | grep 'does not exist'
mkdir -p "$DEDOC_HOME"
wrapped_dedoc ft | grep 'dedoc2'
wrapped_dedoc ls
