#!/bin/sh
# List all pages from a docset, start `fzf`, search for a page and open it in
# `less`.

set -e

if ! which fzf less > /dev/null; then
  echo 'ERROR: Please make sure `fzf` and `less` are available in $PATH.' >&2
  exit 1
fi

D=$1

# Figure out dedoc's path.
REL=./target/release/dedoc
DBG=./target/debug/dedoc

if test -e $DBG; then
  DEDOC="$DBG"
elif test -e $REL; then
  DEDOC="$REL"
else
  DEDOC=`which dedoc`
fi

if test -z $DEDOC; then
  echo 'ERROR: Please make sure `dedoc` is available in $PATH.' >&2
  exit 1
fi

if test -z $1; then
  echo "USAGE"
  echo "    $0 <docset>"
  echo "    Invoke fzf to interactively search a docset via dedoc."
  exit 0
fi

# Make sure the docset we need is installed. Command result substitution below
# doesn't catch errors, so test it manually.
T=`$DEDOC ls --porcelain -l -s=$D`

if test -z $T; then
  echo "ERROR: \`$D\` is not downloaded." >&2
  exit 1
fi

$DEDOC -c open $D `$DEDOC -c ss $D --porcelain | fzf --ansi --layout=reverse --header-lines=1` | less -rR
