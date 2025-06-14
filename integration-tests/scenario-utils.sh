#!/bin/sh

# Utilies for integration tests. This file is not meant to be run directly, but
# rather sourced in other scripts.

export TERM="ansi"
export DEDOC_HOME="/root/.dedoc"
export COVERAGE_DIR="$(realpath "./coverage")"
export LLVM_PROFILE_FILE="$COVERAGE_DIR/profraw/dedoc-%p-%m.profraw"

_log_date() {
date "+%Y-%m-%d at %X"
}

_log_red() {
echo "$(tput setaf 1)$*$(tput sgr0)"
}

_log_bold() {
echo "$(tput bold)$*$(tput sgr0)"
}

log() {
_log_bold "$(printf "$(_log_date) [LOG] %s\n" "$@")" >&2
}

log_err_and_die() {
_log_red "$(printf """$(_log_date) [ERR] %s\n" "$@")" >&2
exit 1
}

diff_stdin_to_text() {
log "Diffing..."
F="$(echo "${1:-"blank"}" | head -n 1 | tr ' ' '_').XXXXXX"
P="$(mktemp -p /tmp "$F")"
if ! test -z "$1"; then
  echo "$1" > "$P"
fi
if ! diff -su "$P" -; then
  log_err_and_die "Stdins differ!"
fi
}

wrapped_cat() {
for F in "$@"; do
  log "Catting $F:"
  printf '```\n'
  cat "$F"
  printf '\n```\n'
done
}

wrapped_dedoc() {
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

CERT_CONFIG_PATH="$(realpath "./data/openssl.cnf")"

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

MOCK_SERVER_PID_PATH="$(realpath "./data/mock_server.pid")"

start_mock_file_server() {
cd data/ || log_err_and_die "No data/, invalid directory structure!"
log "Starting mock file server (key=$KEY_PATH, cert=$CERT_PATH)"
python3 ../https-server.py "127.0.0.1" "443" "$KEY_PATH" "$CERT_PATH" &
PID="$!"
echo "$PID" > "$MOCK_SERVER_PID_PATH"
sleep 2
}

COVERAGE_FILE_PATH="$COVERAGE_DIR/report.md"

generate_coverage_report() {
log "Generating coverage report at $COVERAGE_FILE_PATH..."
grcov --binary-path "../target-docker/x86_64-unknown-linux-musl/debug" \
      -s .. --ignore-not-existing --branch -t markdown \
      --keep-only 'src/*' -o "$COVERAGE_FILE_PATH" "$COVERAGE_DIR/profraw/"
}

swap_docs_json() {
DOCS="$(realpath "$DEDOC_HOME/docs.json")"
DOCS2="$(realpath "./data/docs-2.json")"

mv "$DOCS" "$DOCS.old"
mv "$DOCS2" "$DOCS"
mv "$DOCS.old" "$DOCS2"
}
