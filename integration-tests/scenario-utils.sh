#!/bin/sh

# Utilies for integration tests. This file is not meant to be run directly, but
# rather sourced in other scripts.

_log_date() {
date "+%Y-%m-%d at %X"
}

log() {
  printf "$(_log_date) [LOG] %s\n" "$@" >&2
}

log_err_and_die() {
  printf "$(_log_date) [ERR] %s\n" "$@" >&2
  exit 1
}

diff_to_expected() {
E="expected/$2.out"
log "Diffing $1 and $E..."
if ! diff -su "$1" "$E"; then
  log_err_and_die "$1 and $E differ!"
fi
}

mock_cat() {
for F in "$@"; do
  log "Catting $F:"
  printf '```\n'
  cat "$F"
  printf '\n```\n'
done
}

mock_dedoc() {
DEDOC="$(find ../target-docker/ -name dedoc | head -n 1)"
if test -z "$DEDOC"; then
  log "dedoc binary not found, building..."
  cargo build
fi
log "Running dedoc with arguments: $*..."
$DEDOC --integration-test "$@"
}

override_host() {
if test -z "$1" || test -z "$2"; then
  log_err_and_die "USAGE: override_host <host> <override>"
fi
log "Overriding /etc/hosts ($1 -> $2)..."
printf "$2"'\t'"$1"'\n' >> /etc/hosts
}

KEY_PATH="$(realpath "./data/mock-key.pem")"
CERT_PATH="$(realpath "./data/mock-cert.pem")"

INSTALLED_CERT_PATH="/usr/local/share/ca-certificates/mock-cert.pem"

CERT_CONFIG_PATH="$(realpath "data/openssl.cnf")"

prepare_mock_key() {
log "Preparing mock SSL key ($KEY_PATH, $CERT_PATH)..."

openssl req -x509 -newkey rsa:2048 -nodes \
  -keyout "$KEY_PATH" \
  -out "$CERT_PATH" \
  -days 1 \
  -subj "/CN=devdocs.io" \
  -extensions v3_req \
  -config "$CERT_CONFIG_PATH"
}

install_mock_key() {
log "Installing mock SSL key ($CERT_PATH to $INSTALLED_CERT_PATH)..."
INSTALLED_CERT_DIR="$(dirname "$INSTALLED_CERT_PATH")"
if ! test -d "$INSTALLED_CERT_DIR"; then
  mkdir -p "$INSTALLED_CERT_DIR"
fi
cp "$CERT_PATH" "$INSTALLED_CERT_PATH"
update-ca-certificates
}

make_sure_mock_cert_is_installed() {
if ! test -e "$INSTALLED_CERT_PATH"; then
  prepare_mock_key
  install_mock_key
fi
}

MOCK_SERVER_PID_PATH="$(realpath "data/mock_server.pid")"

mock_server() {
cd data/ || log_err_and_die "No data/, invalid directory structure!"
python3 ../https-server.py "127.0.0.1" "443" "$KEY_PATH" "$CERT_PATH" &
PID="$!"
echo "$PID" > "$MOCK_SERVER_PID_PATH"
}
