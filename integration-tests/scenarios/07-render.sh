#!/bin/sh

# See if dedoc can render docsets.

set -eu
. "$(dirname "$0")"/../scenario-utils.sh

wrapped_dedoc dl docset-1 docset-3~1

wrapped_dedoc render docset-1
diff_stdin_to_text "# test" < "/root/.dedoc/rendered/docset-1/type-1/1.md"

wrapped_dedoc render docset-3~1 -d /root/rendered2
test -e "/root/rendered2/errors/e_mom_yelling.md"

# Can't use already existing directories.
! wrapped_dedoc render docset-3~1 -d /root/rendered2

wrapped_dedoc render --all
test -e "/root/.dedoc/rendered/docset-3~1/errors/e_mom_yelling.md"

wrapped_dedoc render --all -d /root/rendered2/all
test -e "/root/rendered2/all/docset-3~1/errors/e_mom_yelling.md"

wrapped_dedoc rm --purge-all
