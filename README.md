# dedoc

[![Integration Tests](https://github.com/toiletbril/dedoc/actions/workflows/integration-tests.yml/badge.svg?branch=staging)](https://github.com/toiletbril/dedoc/actions/workflows/integration-tests.yml)

Search and view [DevDocs](https://devdocs.io/) offline from your terminal.
**Without browser**. Without Python, Javascript or other inconveniences. Even
without desktop environment.

App directory is `~/.dedoc`. Docsets go into `~/.dedoc/docsets`. You can also
define `$DEDOC_HOME` environment variable to an existing directory of your
choice.

Pages are translated from HTML to colored text (not markdown), and can be piped
to `less`, [`glow`](https://github.com/charmbracelet/glow) if you're fancy, or
any other pager or *maybe* a markdown reader.

If you have Rust, the preferred way to install `dedoc` is by running:
```console
$ cargo install dedoc
```

Alternatively, precompiled `x86_64` binaries for Windows and Linux are
available in [releases](https://github.com/toiletbril/dedoc/releases).

## Development

Everything as in your usual Rust project that uses Cargo.

As for releases, [`Dockerfile`](./Dockerfile) is used as a base image for
`x86_64` Linux/Windows cross-compilation and integration tests. Take a look
inside [`./Shfile.sh`](./Shfile.sh) for more context.

## Usage

Remember that running anything with `--help` prints a more detailed usage:
 ```console
 $ dedoc [subcommand] --help
 ```

To start using `dedoc` and fetch all latest available docsets, first run:
```console
$ dedoc fetch
Fetching `https://devdocs.io/docs.json`...
Writing `/home/user/.dedoc/docs.json`...
Fetching has successfully finished.
```

You can use `-f` flag to overwrite the fetched document if you encounter any
issues.

 To see available docsets, run:
```console
$ dedoc ls
angular, ansible, apache_http_server, astro, async, ...
```

Which will list all docsets available to download from file which you
previously fetched. If you need version-specific docs, like `vue~3`/`~2`, use
`-a` flag, which will list *everything*.

Using `-l` flag will show only local docsets, and `-n` will print each docset
on a separate line.

Download the documentation:
```console
$ dedoc download rust
Downloading `rust`...
Received 46313067 bytes, file 1 of 2...
Received 3319078 bytes, file 2 of 2...
Extracting to `/home/user/.dedoc/docsets/rust`...
Unpacked 1899 files...
Install has successfully finished.
```

This will make the documentation available locally as a bunch of HTML pages.

You can use `-f` flag here too to forcefully overwrite the documentation.

To search, for instance, for `BufReader` from `rust`, run:
```console
$ dedoc search rust bufreader
Searching for `bufreader`...
Exact matches in `rust`:
   1  std/io/struct.bufreader
         2  #method.borrow
         3  #method.borrow_mut
         4  #method.buffer
         5  #method.by_ref
         ...
```

You will get search results which are pages that match your query.

Results that start with `#` denote fragments. Opening them will result in the
output of only that specific fragment. Likewise, opening a page will show the
entire page. If you want to forcefully print the entire page instead of only a
fragment, use `-f` flag.

For a more detailed search, use the `-p` flag. It makes search behave similarly
to the `grep` command, and will look within all files, find all matches, and
display them with some context around the found section.

Use `-i` to perform case-insensitive search, and `-w` to search for the whole
sentence.

Finally, to see the page, you can run `open` with the path with optional
fragment:
```console
$ dedoc open rust "std/io/struct.bufreader#method.borrow"
...
fn borrow(&self) -> &T
Immutably borrows from an owned value. Read more
source
...
```

Using `-h` with `open` makes `dedoc` interpret supplied arguments as a path to
HTML file and behave like a HTML to text transpiler. To make output wider or
narrower, you can use `-c` flag with the number of columns.

Instead of typing out the whole path, you can conveniently append `-o` flag the
your previous `search` command, which will open n-th matched page or fragment:
```console
$ dedoc search rust bufreader -o 2
```

This will be as fast as `open`, due to search caching. `-c` flag here works the
same way as in `open`.

You would probably like to forcefully enable colors for non-terminals with `-c`,
use `ss` instead of `search` and pipe output to a pager or markdown reader, like
`less` with `-r` to reinterpret colors, turning the final command into:
```console
$ dedoc -c ss rust bufreader -o 2 | less -r
```

## Scripting support

There is a `render` subcommand, which allows you to render the entire docset to
text. By default, all docsets are stored in HTML files and are rendered on the
fly, to support toggling the colors and dynamic output size. By using the
subcommand, the docset will be rendered without colors and with the width
specified in `-c` (144 by default) into a directory specified in `-d`
(`~/.dedoc/rendered/<docset>` by default). You can render all at once with
`--all` and re-render as much as you want.

Some commands support `--porcelain`, to make life slightly easier when parsing
the output.

You may take a look at the [example script](./dedoc-fzf.sh) for an inspiration.

Happy coding!
