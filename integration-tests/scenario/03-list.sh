#!/bin/sh

# See if dedoc can successfully list things.

set -eu
. "$(dirname "$0")"/../scenario-utils.sh

mock_dedoc ls | mock_diff_stdin_to_text "docset-1, docset-2"
mock_dedoc ls -a | \
mock_diff_stdin_to_text "docset-1, docset-2, docset-3~1, docset-3~2"
mock_dedoc ls -l | mock_diff_stdin_to_text ""
mock_dedoc ls -o | mock_diff_stdin_to_text "docset-1, docset-2"

mock_dedoc dl "docset-1" "docset-3~1"

mock_dedoc ls | mock_diff_stdin_to_text "docset-1 [downloaded], docset-2"

mock_dedoc ls -a | \
mock_diff_stdin_to_text "docset-1 [downloaded], docset-2, docset-3~1 [downloaded], docset-3~2"

mock_dedoc ls -l | \
mock_diff_stdin_to_text "docset-1 [downloaded], docset-3~1 [downloaded]"
mock_dedoc ls -la | \
mock_diff_stdin_to_text "docset-1 [downloaded], docset-3~1 [downloaded]"

mock_dedoc ls -ld | mock_diff_stdin_to_text "docset-1, docset-3~1"
mock_dedoc ls -lad | mock_diff_stdin_to_text "docset-1, docset-3~1"

mock_dedoc ls -o | mock_diff_stdin_to_text "docset-2"
mock_dedoc ls -oa | mock_diff_stdin_to_text "docset-2, docset-3~2"

mock_dedoc ls -da | \
mock_diff_stdin_to_text "docset-1, docset-2, docset-3~1, docset-3~2"

mock_dedoc ls --search="3" | \
mock_diff_stdin_to_text "docset-3~1 [downloaded], docset-3~2"

mock_dedoc rm --purge-all
