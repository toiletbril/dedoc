#!/bin/sh

# Tests for the install command.

set -eu
. "$(dirname "$0")"/../scenario-utils.sh

wrapped_dedoc install | tail # script warning

D="$(realpath "$(dirname "$(which_dedoc)")")"

stat "$D/dedoc-interactive" && log_err_and_die "script should not exist yet"

wrapped_dedoc install --accept
head -n 1 < "$D"/dedoc-interactive # shebang :3

rm "$D"/dedoc-interactive
