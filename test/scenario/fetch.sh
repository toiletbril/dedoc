#!/bin/sh

# See if dedoc can successfully fetch mock dock.json.

set -eu
. "$(dirname "$0")"/../scenario-utils.sh

mock_https_response 443 docs.json &
sleep 1
mock_dedoc fetch
