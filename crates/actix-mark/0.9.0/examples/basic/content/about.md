# About

`actix_mark` is a Rust library that makes it easy to serve Markdown documentation
alongside your actix-web application.

## How it works

1. You point `MarkdownFiles` at a directory.
2. Requests for `.md` files (or extension-less URLs) are matched, the Markdown
   is converted to HTML with [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark).
3. The HTML is injected into your template at the `<markdown>` tag.
4. Non-Markdown files (CSS, images…) are served unchanged.

[← Back to index](.)
