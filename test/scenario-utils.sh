#!/bin/sh

# Utilies for integration tests. This file is not meant to be run directly, but
# rather sourced in other scripts.

log() {
  printf "$(date "+%x,%X"),LOG,%s\n" "$@" >&2
}

log_err_and_die() {
  printf "$(date "+%x,%X"),ERR,%s\n" "$@" >&2
  exit 1
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

KEY_PATH="data/mock-key.pem"
CERT_PATH="data/mock-cert.pem"
CERT_CONFIG_PATH="data/cert-config"
INSTALLED_CERT_PATH="/usr/local/share/ca-certificates/mock-cert.pem"

prepare_mock_key() {
log "Preparing mock SSL key ($KEY_PATH, $CERT_PATH)..."

cat > "$CERT_CONFIG_PATH" <<EOF
[req]
distinguished_name = req_distinguished_name
x509_extensions = v3_req
prompt = no

[req_distinguished_name]
CN = devdocs.io

[v3_req]
basicConstraints = critical, CA:FALSE
keyUsage = critical, digitalSignature, keyEncipherment
extendedKeyUsage = serverAuth
subjectAltName = @alt_names

[alt_names]
DNS.1 = devdocs.io
EOF

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

mock_https_response() {
if test -z "$1" || test -z "$2"; then
  log_err_and_die "USAGE: mock_https_response <port> data/<file>"
fi

PORT="$1"
FILE="data/$2"
KEY="$KEY_PATH"
CERT="$CERT_PATH"

if test ! -f "$KEY" || ! test -f "$CERT"; then
  log_err_and_die "Certificate files not found. Run make_sure_mock_cert_is_installed first."
fi

BODY=$(cat "$FILE")
BODY_LENGTH=$(printf "%s" "$BODY" | wc -c)

RESPONSE="\
HTTP/1.1 200 OK\r
Content-Type: application/json\r
Content-Length: $BODY_LENGTH\r
\r
$BODY"

log "Mocking HTTPS response on port $PORT with contents of $FILE..."
printf "%b" "$RESPONSE" | openssl s_server \
  -quiet \
  -key "$KEY" \
  -cert "$CERT" \
  -accept "$PORT" \
  -naccept 1
}
