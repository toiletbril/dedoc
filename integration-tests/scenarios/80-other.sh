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
wrapped_dedoc il --help

wrapped_dedoc --color=off
wrapped_dedoc --color=auto

# Invalid flags.
wrapped_dedoc ls -ns && log_err_and_die "combined flags should be invalid"
wrapped_dedoc ls -s && log_err_and_die "no value provided should be invalid"
wrapped_dedoc --color=2 && log_err_and_die "2 should be an invalid argument"
wrapped_dedoc --aaaa && log_err_and_die "there should be no -a flag"

# Program directory should be created if DEDOC_HOME is not set.
rm -rf "$DEDOC_HOME"
DEDOC_HOME_BAK="$DEDOC_HOME"
wrapped_dedoc ft && log_err_and_die "directory does not exist"
unset DEDOC_HOME
wrapped_dedoc ft
export DEDOC_HOME="$DEDOC_HOME_BAK"

wrapped_dedoc -W dl docset-1 &
sleep 1
kill -s STOP %%
# another instance is downloading the same docset
wrapped_dedoc dl docset-1 && log_err_and_die "concurrent download of same docset should block"
wrapped_dedoc op docset-1 something && log_err_and_die "open should be blocked too"
kill -s CONT %%
wait %%
wrapped_dedoc dl docset-1 # success

# creates a per-docset lock file, does not delete it (incomplete state)
wrapped_dedoc -WW dl docset-1
cat "$DEDOC_HOME/docsets/docset-1/.lock"
wrapped_dedoc dl docset-1 && log_err_and_die "non-force download should refuse incomplete docset"
wrapped_dedoc dl -f docset-1 # -f detects stale lock, deletes incomplete docset, re-downloads

echo "what" > "$DEDOC_HOME/docsets/docset-1/.lock"
wrapped_dedoc dl docset-1 && log_err_and_die "bogus pid is treated as stale, should refuse"
wrapped_dedoc dl -f docset-1 # re-downloads with bogus stale lock
