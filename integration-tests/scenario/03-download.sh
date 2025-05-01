#!/bin/sh

# See if dedoc can successfully download things.

set -eu
. "$(dirname "$0")"/../scenario-utils.sh

mock_dedoc dl docset-1 docset-3~1
