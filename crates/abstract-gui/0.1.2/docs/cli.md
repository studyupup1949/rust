# CLI

## Commands

```sh
gui check examples/demo.gui
gui check examples/demo.gui other.gui
gui check

gui page examples/demo.gui
gui page
gui drill
gui inherit
gui node
gui nav

gui scan page1.html page2.html
```

## Input resolution

If file arguments are omitted, `gui` recursively scans the current working
directory and uses the union of all matching `*.gui` files.

If multiple `.gui` files are given explicitly, they are merged into one logical
document before the command runs.

## Command summary

- `check`: parse and validate `.gui` input
- `page`: list nodes that satisfy the current page rules
- `drill`: print the `drill` tree with indentation
- `inherit`: print the `inherit` tree with indentation
- `node`: list node ids
- `nav`: list nav ids
- `scan`: infer `.gui` from rendered HTML files and print to stdout

## Typical workflows

Validate one file:

```sh
gui check examples/demo.gui
```

Validate all `.gui` files in the current tree:

```sh
gui check
```

Inspect page ids:

```sh
gui page
```

Scan rendered HTML into `.gui`:

```sh
gui scan saved/home.html saved/pricing.html > site.gui
```

## Notes

- `gui scan` does not fetch pages or execute JavaScript.
- HTML acquisition is intentionally out of scope for the CLI.
