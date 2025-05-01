#!/bin/sh

# See if dedoc can successfully list things.

set -eu
. "$(dirname "$0")"/../scenario-utils.sh

mock_dedoc ls | diff_to_expected - "ls"
mock_dedoc ls -a | diff_to_expected - "ls-all"
