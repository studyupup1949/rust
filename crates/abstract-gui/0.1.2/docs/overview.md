# Overview

`abstract-gui` models GUI structure as a small set of orthogonal relations.

## Core ideas

- `drill`: semantic drill-down in information space
- `inherit`: shared layout, navigation, and capability inheritance
- `nav`: named sets of target pages
- `node`: attributed GUI nodes

This separation lets the model say:

- what a page drills into
- what a page shares with other pages
- what shared navigators can reach
- what attributes belong to a node

## Page rules

- every `inherit` leaf is a page
- every node appearing in `drill` is a page
- non-leaf `inherit` nodes may instead be layouts, shells, or templates

## Attribute rules

- scalar attributes override inherited values
- vector attributes merge by set union
- `nav` is unordered at the language level

Concrete UI layers may still choose tabs, sidebars, menus, rings, or any other
presentation.

## Scan output kinds

`gui scan` currently emits these node kinds:

- `page`
- `section`
- `layout`
- `action`
- `index`
- `dialog`

Dialog nodes may also carry `dialog-kind` such as `form`, `confirm`, or `promo`.

## Related docs

- [CLI](./cli.md)
- [Scanning HTML](./scan.md)
- [Roadmap](./roadmap.md)
