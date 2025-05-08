#!/bin/sh

# See if dedoc can successfully list things.

set -eu
. "$(dirname "$0")"/../scenario-utils.sh

# Shows only non-donwloaded, non-versioned docsets.
wrapped_dedoc ls | diff_stdin_to_text "docset-1, docset-2"

# Shows all non-donwloaded docsets.
wrapped_dedoc ls -a | \
diff_stdin_to_text "docset-1, docset-2, docset-3~1, docset-3~2"

# Shows local docsets, nothing in this case.
wrapped_dedoc ls -l | diff_stdin_to_text ""

# Shows non-local docsets, same as ordinary "ls", since no docsets been
# downloaded.
wrapped_dedoc ls -o | diff_stdin_to_text "docset-1, docset-2"

wrapped_dedoc dl "docset-1" "docset-3~1"

# Shows [downloaded] label around docset-1. docset-3~1 is versioned and not
# shown here.
wrapped_dedoc ls | diff_stdin_to_text "docset-1 [downloaded], docset-2"

# Shows [downloaded] label around docset-1 and docset-3~1.
wrapped_dedoc ls -a | \
diff_stdin_to_text "docset-1 [downloaded], docset-2, docset-3~1 [downloaded], docset-3~2"

# Shows downloaded docsets only. -l implies -a.
wrapped_dedoc ls -l | \
diff_stdin_to_text "docset-1 [downloaded], docset-3~1 [downloaded]"
wrapped_dedoc ls -la | \
diff_stdin_to_text "docset-1 [downloaded], docset-3~1 [downloaded]"

# Local docsets without [downloaded] label.
wrapped_dedoc ls -ld | diff_stdin_to_text "docset-1, docset-3~1"
wrapped_dedoc ls -lad | diff_stdin_to_text "docset-1, docset-3~1"

# Online docsets without version.
wrapped_dedoc ls -o | diff_stdin_to_text "docset-2"
# Online docsets with version.
wrapped_dedoc ls -oa | diff_stdin_to_text "docset-2, docset-3~2"

# All docsets without labels.
wrapped_dedoc ls -da | \
diff_stdin_to_text "docset-1, docset-2, docset-3~1, docset-3~2"

# --search, implies -a.
wrapped_dedoc ls --search="3" | \
diff_stdin_to_text "docset-3~1 [downloaded], docset-3~2"

# -n.
wrapped_dedoc ls -an | \
diff_stdin_to_text \
'docset-1 [downloaded]
docset-2
docset-3~1 [downloaded]
docset-3~2'

wrapped_dedoc rm --purge-all
