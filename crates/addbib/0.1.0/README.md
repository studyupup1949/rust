# addbib

An opinionated cli app that adds a linked bibliography to markdown documents,
where the links are realised with html.

For available commands, please run `addbib --help`.

**This app is not designed for production use, but for a personal need in
wanting linked citations in my Zettelkasten that is composed of markdown
files.**

Please always backup your markdown files, and test with `--dryrun`, before using
the app on any markdown.

## Usage

The intention for this app is to be part of a pipeline that generates html pages
from markdown notes.

An example scenario would be a folder of markdown files (for instance a
[Zettelkasten](https://en.wikipedia.org/wiki/Zettelkasten)), where each markdown
file is intended to generate a html page. Some example apps that convert
markdown files to html pages are [quartz](https://quartz.jzhao.xyz/) and
[mkdocs](https://www.mkdocs.org/). This app will add a bibliography to the
markdown files, where in-text citations will be replaced with internal html
links, which direct the user to the related bibliography item at the end of the
markdown file. The links are realised with html.

In more explicit terms, given a .md, .bib and .csl file, the app adds a linked
bibliography to the end of the markdown file, and inserts html into the markdown
file to realise this. Please note, that the .csl file is **only used to format
the bibliography, not the citations. The citations are always numbered and are
always presented in square brackets.**

Valid markdown citation keys (that the app will find) are those that start with
the `@` symbol, and/or closed within square brackets, for instance:

- `@Author2025`
- `[@Author2025]`
- `[@AuthorOne2025, @AuthorTwo2025, ..]`

There are various options in using the app, such as:

- not producing any html links (`--nohtml`) where only the bibliography and
  numbered citations are added,
- output to different files (`-o`, `--output`) instead of overwritting the
  markdown file,
- or simply perform a `--dryrun` that not change anything, and show all changes
  that would have occurred in the terminal.

## Installation

```bash
cargo install addbib
```

## Contributions

Contributions are more than welcome!

I suspected the most wanted change would be to allow the csl file to style the
citation keys. I will see to adding this feature if there's interest from
others. I would also document the code, and organise the repository better in
general.

## Licence

addbib is licensed under the EUPL-1.2 or later. You may obtain a copy of the
licence at <https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12>. A
copy of the EUPL-1.2 licence in English is given in [LICENSE.txt](LICENSE.txt)
which is found in the root of this repository.
