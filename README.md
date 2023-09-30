# dedoc

Search [DevDocs](https://devdocs.io/) from your terminal. Offline. **Without
browser**. Without Python, Javascript or other inconveniences. Even without
desktop environment.

App directory is `~/.dedoc`. Docsets go into `~/.dedoc/docsets`.

Pages are displayed as markdown documents, and can be piped to `less`,
[`glow`](https://github.com/charmbracelet/glow) if you're fancy, or any other
pager or markdown reader.

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
entire page.

For a more detailed search, use the `-p` flag. It makes search behave similarly
to the `grep` command, and will look within all files, find all matches, and
display them with some context around the found section.

Use `-i` to perform case-insensitive search, and `-w` to search for the whole
sentence.

Finally, to see the page, you can run `open`:
```console
$ dedoc open rust std/io/struct.bufreader
```

Use `-h` flag with `open` to make `dedoc` behave like a HTML to markdown
transpiler. It will interpret supplied arguments as a path to HTML file.

More conveniently, append `-o` flag the your previous `search` command, which
will open n-th matched page or fragment:
```console
$ dedoc search rust bufreader -o 1
```

This will be as fast as `open`, due to search caching.

You would probably like to use `ss` instead of `search`, pipe output to a pager
or markdown reader, like `less` and forcefully enable colors with `-c y`,
turning the final command into:
```console
$ dedoc -c y ss rust bufreader -o 1 | less -r
```

Happy coding!
