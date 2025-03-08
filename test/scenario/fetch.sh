#!/bin/sh

set -eu

. `dirname "$0"`/../scenario_utils.sh

override_host devdocs.io 127.0.0.1
mock_response 443 docs.json &

mock_dedoc fetch
echo $?
