#!/bin/sh

# See if dedoc can successfully download things.

set -eu
. "$(dirname "$0")"/../scenario-utils.sh

wrapped_dedoc dl docset-1 docset-3~1

# Everything is up to date.
! wrapped_dedoc dl -u

# Non-existent docset.
! wrapped_dedoc dl whatever

# Break .mtime files, so dedoc would have to update both docsets.
rm "$DEDOC_HOME/docsets/docset-1/.dedoc_mtime"
printf "0" > "$DEDOC_HOME/docsets/docset-3~1/.dedoc_mtime"

wrapped_dedoc dl -u | grep "2 items were successfully updated."

# Only one docset should get updated.
rm "$DEDOC_HOME/docsets/docset-3~1/.dedoc_mtime"

wrapped_dedoc dl -u | grep "1 item was successfully updated."

# Swap mock docs.json to confuse dedoc with non-existing docsets on update.
swap_docs_json
wrapped_dedoc dl -u 2>&1 | grep "Unable to find"
swap_docs_json

wrapped_dedoc rm --purge-all
