#!/bin/sh

# See if dedoc can remove things.

set -eu
. "$(dirname "$0")"/../scenario-utils.sh

# Download docset and remove it. Nothing remains.
wrapped_dedoc dl docset-1
wrapped_dedoc rm docset-1
wrapped_dedoc ls -l | diff_stdin_to_text ""

# Download two docsets and remove one. One remains.
wrapped_dedoc dl docset-1 docset-3~1
wrapped_dedoc rm docset-1
wrapped_dedoc ls -l | diff_stdin_to_text "docset-3~1 [downloaded]"

# Download missing docset and remove them both. Nothing remains.
wrapped_dedoc dl docset-1
wrapped_dedoc rm docset-1 docset-3~1
wrapped_dedoc ls -l | diff_stdin_to_text ""

# Download both docsets and purge everything. Nothing remains.
wrapped_dedoc dl docset-1 docset-3~1
wrapped_dedoc rm --purge-all
wrapped_dedoc ls -l | diff_stdin_to_text ""
