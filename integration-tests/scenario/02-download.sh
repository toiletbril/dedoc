#!/bin/sh

# See if dedoc can successfully download things.

set -eu
. "$(dirname "$0")"/../scenario-utils.sh

mock_dedoc dl docset-1 docset-3~1

# Everything is up to date.
! mock_dedoc dl -u

# Non-existent docset.
! mock_dedoc dl whatever

# Break .mtime files, so dedoc would have to update both docsets.
rm "$DEDOC_HOME/docsets/docset-1/.dedoc_mtime"
printf "0" > "$DEDOC_HOME/docsets/docset-3~1/.dedoc_mtime"

mock_dedoc dl -u | grep "2 items were successfully updated."

# Only one docset should get updated.
rm "$DEDOC_HOME/docsets/docset-3~1/.dedoc_mtime"

mock_dedoc dl -u | grep "1 item was successfully updated."

mock_dedoc rm --purge-all
