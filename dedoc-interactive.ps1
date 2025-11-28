#!/usr/bin/env powershell

# Interactive docset browser via `dedoc`.
#
# List all pages from a docset, start `skim`, search for a page and open it in
# `moar`.

function get-exists()
{
  param
  (
    [Parameter(Position=0)]
    [string]$Path
  )

  return get-command $Path -ErrorAction SilentlyContinue
}

# Find " FZF.
if (get-exists "skim")
{
  $Fzf = 'skim'
}
elseif (get-exists "sk")
{
  $Fzf = 'sk'
}
elseif (get-exists "fzf")
{
  $Fzf = 'fzf'
}
else
{
  write-output 'ERROR: Please make sure `skim` is available in $PATH.'
  exit 1
}

# Adjust the layout.
$Fzf = "$Fzf --ansi --layout=reverse"

if (get-exists 'moar')
{
  $Pager = 'moar'
}
elseif (get-exists "less")
{
  $Pager = 'less -R'
}
else
{
  write-output 'ERROR: Please make sure `moar` is available in $PATH.'
  exit 1
}

# Figure out dedoc's path.
$DedocRel = './target/release/dedoc'
$DedocDbg = './target/debug/dedoc'

if (get-exists "$DedocRel")
{
  $Dedoc = "$DedocRel"
}
elseif (get-exists "$DedocDbg")
{
  $Dedoc = "$DedocDbg"
}
elseif (get-exists 'dedoc')
{
  $Dedoc = get-command 'git'
}
else
{
  write-output 'ERROR: Please make sure `dedoc` is available in $PATH.'
  exit 1
}

while ($true)
{
  $Docset = "$(invoke-expression "dedoc ls -l --porcelain | $Fzf")"
  if (!$Docset)
  {
    break
  }

  while ($true)
  {
    $Page = "$(invoke-expression "$Dedoc -c ss $DOCSET --porcelain | $Fzf" -ErrorAction Ignore)"
    if (!$Page)
    {
      break
    }

    invoke-expression "$Dedoc -c open $Docset $Page | $Pager"
  }
}
