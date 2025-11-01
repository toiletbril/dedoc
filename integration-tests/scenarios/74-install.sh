#!/bin/sh

# Tests for the install command.

set -eu
. "$(dirname "$0")"/../scenario-utils.sh

wrapped_dedoc | tail # script warning

wrapped_dedoc install --accept
head -n 1 < "$DEDOC_HOME"/dedoc-interactive # shebang :3

rm "$DEDOC_HOME"/dedoc-interactive
