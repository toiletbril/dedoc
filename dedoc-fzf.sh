#!/bin/sh

# List all pages from a docset, start `fzf`, search for a page and open it in
# `less`.

set -eu

# Find fzf.
if ! which fzf > /dev/null; then
  echo 'ERROR: Please make sure `fzf` is available in $PATH.' >&2
  exit 1
fi

# Choose the pager.
if which moar > /dev/null; then
  PAGER=moar
elif which less > /dev/null; then
  PAGER='less -R'
else
  echo 'ERROR: Please make sure `less` or `moar` is available in $PATH.' >&2
  exit 1
fi

# Figure out dedoc's path.
REL=./target/release/dedoc
DBG=./target/debug/dedoc

if test -e "$REL"; then
  DEDOC="$REL"
elif test -e "$DBG"; then
  DEDOC="$DBG"
else
  DEDOC="$(which dedoc)"
fi

if test -z "$DEDOC"; then
  echo 'ERROR: Please make sure `dedoc` is available in $PATH.' >&2
  exit 1
fi

if test -z "${1:-}"; then
  echo "USAGE"
  echo "    $0 <docset>"
  echo "    Invoke fzf to interactively search a docset via dedoc."
  exit 0
fi

DOCSET="$1"

# Make sure the docset we need is installed. Command result substitution below
# doesn't catch errors, so test it manually.
"$DEDOC" ls -l --exists="$DOCSET" || exit 1

$DEDOC -c open "$DOCSET" \
"$("$DEDOC" -c ss "$DOCSET" --porcelain | \
   fzf --ansi --layout=reverse --header-lines=1)" | \
$PAGER
