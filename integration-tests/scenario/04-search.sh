#!/bin/sh

# See if dedoc can search downloaded docsets.

set -eu
. "$(dirname "$0")"/../scenario-utils.sh

mock_dedoc ss docset-1 | diff_to_expected - "search-docset-1"
mock_dedoc ss docset-1 -o 1 | diff_to_expected - "open-docset-1-type-1-1"
mock_dedoc ss docset-1 -o 1 -n | diff_to_expected - "open-docset-1-type-1-1-n"
