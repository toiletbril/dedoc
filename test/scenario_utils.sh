#!/bin/sh

mock_dedoc() {
DEDOC=`find ../target-docker/ -name dedoc | head -n 1`
if test -z "$DEDOC"; then
  cargo build
fi
$DEDOC --integration-test "$@"
}

override_host() {
if test -z "$1" || test -z "$2"; then
  echo "USAGE: override_host <host> <override>" >&2
  return
fi
printf "$2"'\t'"$1"'\n' >> /etc/hosts
}

mock_response() {
if test -z "$1" || test -z "$2"; then
  echo "USAGE: mock_response <port> <file>" >&2
  return
fi

PORT=$1
FILE=$2

RESPONSE="\
HTTP/1.1 200 Success
Content-Type: application/json
Content-Length: $(wc -c "$FILE")

$(cat "$FILE")"

echo "$RESPONSE" | nc -l -p "$PORT" -q 0 >&2
}
