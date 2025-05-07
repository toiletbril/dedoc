#!/bin/sh

# See if dedoc can remove things.

set -eu
. "$(dirname "$0")"/../scenario-utils.sh

mock_dedoc dl docset-1
mock_dedoc rm docset-1
mock_dedoc ls -l | mock_diff_stdin_to_text ""

mock_dedoc dl docset-1 docset-3~1
mock_dedoc rm docset-1
mock_dedoc ls -l | mock_diff_stdin_to_text "docset-3~1 [downloaded]"

mock_dedoc dl docset-1
mock_dedoc rm docset-1 docset-3~1
mock_dedoc ls -l | mock_diff_stdin_to_text ""

mock_dedoc dl docset-1 docset-3~1
mock_dedoc rm --purge-all
mock_dedoc ls -l | mock_diff_stdin_to_text ""
