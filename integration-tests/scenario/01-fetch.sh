#!/bin/sh

# See if dedoc can successfully fetch mock dock.json.

set -eu
. "$(dirname "$0")"/../scenario-utils.sh

mock_dedoc fetch
mock_cat ~/.dedoc/docs.json
