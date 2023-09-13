# dedoc

Search [DevDocs](https://devdocs.io/) from your terminal. Offline. **Without
browser**. Without Python, Javascript or other inconveniences. Even without
desktop environment.

App directory is `~/.dedoc`. Docsets go into `~/.dedoc/docsets`.

Pages are displayed as markdown documents, and can be piped to `less`,
[`glow`](https://github.com/charmbracelet/glow) if you're fancy, or any other
pager or markdown reader.

## Usage

1. To start using `dedoc` and fetch all latest available docsets, first run:
```console
$ dedoc fetch
Fetching `https://devdocs.io/docs.json`...
Writing `docs.json`...
Successfully updated `docs.json`.
```

You can use `-f` flag to overwrite the fetched document if you encounter any issues.

2. To see available docsets, run:
```console
$ dedoc ls
angular, ansible, apache_http_server, astro, async, ...
```

Which will list all docsets available to download from file which you
previously fetched. If you need version-specific docs, like vue~3 or ~2, use
`-a` flag, which will list *everything*.

3. Download the documentation:
```console
$ dedoc download rust
Downloading `rust`...
Extracting `rust`...
Successfully installed `rust`.
```

This will make the documentation available locally as a bunch of HTML pages.

4. To search, for instance, for `BufReader` from `rust`, run:
```console
$ dedoc search rust bufreader
Exact matches in `rust`:
  std/io/struct.bufreader
```

You will get search results which are pages with filenames that match your
query. If you need a more thorough search, you can use `-p` flag, which will
look inside of the files as well.

5. Finally, to see the page you can either use `dedoc open`:
```console
$ dedoc open rust std/io/struct.bufreader
```

Or use `-o` flag, which will open n-th matched page:
```console
$ dedoc search rust bufreader -o 1
```

You would probably like to use `s` instead of `search`, pipe output to a pager,
like `less` and forcefully enable colors if your pager supports it, which turns
the final command into:
```console
$ dedoc -c on s rust bufreader -o 1 | less
```

## Help

```console
$ dedoc --help
USAGE
    dedoc <subcommand> [args]
    Search DevDocs pages from terminal.

SUBCOMMANDS
    fetch                           Fetch available docsets.
    list                            Show available docsets.
    download                        Download docsets.
    remove                          Delete docsets.
    search                          List pages that match your query.
    open                            Display specified pages.

OPTIONS
    -c, --color <on/off/auto>       Use color when displaying output.
    -v, --version                   Display version.
        --help                      Display help message. Can be used with subcommands.
```
