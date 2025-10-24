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

wrapped_dedoc -W ls &
sleep 1
kill -s STOP %%
# another instance is running
wrapped_dedoc ls && log_err_and_die "another instance should block execution"
kill -s CONT %%
wait %%
wrapped_dedoc ls # success

# creates a lock file, does not delete it
wrapped_dedoc -WW ls
cat "$DEDOC_HOME/.lock"
wrapped_dedoc ls # success, even though lock file exists, the owning process is dead

echo "what" > "$DEDOC_HOME/.lock"
wrapped_dedoc ls # success, bogus pid is ignored
