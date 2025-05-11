#!/bin/sh

# Miscellaneous tests.

set -eu
. "$(dirname "$0")"/../scenario-utils.sh

# For manual inspection (and more percentage in code coverage =D).
wrapped_dedoc -v
wrapped_dedoc -V
wrapped_dedoc -c

wrapped_dedoc --help
wrapped_dedoc ft --help
wrapped_dedoc ls --help
wrapped_dedoc rm --help
wrapped_dedoc dl --help
wrapped_dedoc ss --help
wrapped_dedoc op --help
wrapped_dedoc rr --help

wrapped_dedoc --color=off
wrapped_dedoc --color=auto

# Invalid flags.
! wrapped_dedoc ls -ns
! wrapped_dedoc ls -s
! wrapped_dedoc --color=2
! wrapped_dedoc --aaaa

# Program directory should be created if DEDOC_HOME is not set.
rm -rf "$DEDOC_HOME"
DEDOC_HOME_BAK="$DEDOC_HOME"
! wrapped_dedoc ft
unset DEDOC_HOME
wrapped_dedoc ft
export DEDOC_HOME="$DEDOC_HOME_BAK"
