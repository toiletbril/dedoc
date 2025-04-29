#!/bin/sh

# Setup script, ran before any of the scenarios.

set -eu
. "$(dirname "$0")"/scenario-utils.sh

override_host devdocs.io 127.0.0.1
make_sure_mock_cert_is_installed
