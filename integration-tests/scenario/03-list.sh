#!/bin/sh

# See if dedoc can successfully list things.

set -eu
. "$(dirname "$0")"/../scenario-utils.sh

# Shows only non-donwloaded, non-versioned docsets.
mock_dedoc ls | mock_diff_stdin_to_text "docset-1, docset-2"

# Shows all non-donwloaded docsets.
mock_dedoc ls -a | \
mock_diff_stdin_to_text "docset-1, docset-2, docset-3~1, docset-3~2"

# Shows local docsets, nothing in this case.
mock_dedoc ls -l | mock_diff_stdin_to_text ""

# Shows non-local docsets, same as ordinary "ls", since no docsets been
# downloaded.
mock_dedoc ls -o | mock_diff_stdin_to_text "docset-1, docset-2"

mock_dedoc dl "docset-1" "docset-3~1"

# Shows [downloaded] label around docset-1. docset-3~1 is versioned and not
# shown here.
mock_dedoc ls | mock_diff_stdin_to_text "docset-1 [downloaded], docset-2"

# Shows [downloaded] label around docset-1 and docset-3~1.
mock_dedoc ls -a | \
mock_diff_stdin_to_text "docset-1 [downloaded], docset-2, docset-3~1 [downloaded], docset-3~2"

# Shows downloaded docsets only. -l implies -a.
mock_dedoc ls -l | \
mock_diff_stdin_to_text "docset-1 [downloaded], docset-3~1 [downloaded]"
mock_dedoc ls -la | \
mock_diff_stdin_to_text "docset-1 [downloaded], docset-3~1 [downloaded]"

# Local docsets without [downloaded] label.
mock_dedoc ls -ld | mock_diff_stdin_to_text "docset-1, docset-3~1"
mock_dedoc ls -lad | mock_diff_stdin_to_text "docset-1, docset-3~1"

# Online docsets without version.
mock_dedoc ls -o | mock_diff_stdin_to_text "docset-2"
# Online docsets with version.
mock_dedoc ls -oa | mock_diff_stdin_to_text "docset-2, docset-3~2"

# All docsets without labels.
mock_dedoc ls -da | \
mock_diff_stdin_to_text "docset-1, docset-2, docset-3~1, docset-3~2"

# --search, implies -a.
mock_dedoc ls --search="3" | \
mock_diff_stdin_to_text "docset-3~1 [downloaded], docset-3~2"

# -n.
mock_dedoc ls -an | \
mock_diff_stdin_to_text \
'docset-1 [downloaded]
docset-2
docset-3~1 [downloaded]
docset-3~2'

mock_dedoc rm --purge-all
