# dedoc

Search [DevDocs](https://devdocs.io/) from your terminal. Offline. Without browser.
**Without Python, Javascript or other inconveniences**. Even without desktop environment.

App directory is `~/.dedoc`. Docsets go into `~/.dedoc/docsets`.

Pages are displayed as Markdown strings using amazing
[html2text](https://github.com/jugglerchris/rust-html2text/) library, and can
be piped to `less` or something alike.

## Usage

1. To start using `dedoc` and fetch all latest available docsets, first run:
```console
$ dedoc fetch
Fetching `https://devdocs.io/docs.json`...
Writing `docs.json`...
Successfully updated `docs.json`.
```

2. To see available docsets, run:
```console
$ dedoc ls
angular, ansible, apache_http_server, astro, async, ...
```

Which will list all docsets available to download from file which you
previously fetched. If you need version-specific docs, use `-a` flag, which
will list *everything*.

3. Download the documentation:
```console
$ dedoc download rust
Downloading `rust`...
Extracting `rust`...
Successfully installed `rust`.
```

This will make the documentation available locally as a bunch of HTML pages.

4. To search in these pages, for BufReader, as an example, run:
```console
$ dedoc search rust bufreader
Exact matches in `rust`:
  std/io/struct.bufreader
```

You will get search results which are pages with filenames that match your
query. If you need a more thorough search, you can use `-p` flag, which will
look inside of files as well.

5. Finally, to see the page:
```console
$ dedoc read rust std/io/struct.bufreader
```

## Help

```console
$ dedoc --help
USAGE
    dedoc <subcommand> [args]
    Search DevDocs pages from terminal.

SUBCOMMANDS
    fetch              Fetch latest available docsets.
    list               Show available docsets.
    download           Download a docset.
    search             List pages that match your query.
    read               Display the specified page.

    Subcommands can be used by their first letter, for example:
    dedoc s rust -i bufreader

OPTIONS
    --help,                Display help message. Can be used with subcommands.
    --version              Display version.

The design is not final, and may be subject to change.
```
