An Access Control Abstract for Caitsith 

## About

ACLNeko is a Rust binding for [caitsith](https://caitsith.sourceforge.io) policy language.

## Features

* Single `Acl` struct for a valid bundle of caitsith's policy blocks and rules. You can list, search, add, remove, merge, unmerge or report statics of policy blocks with its simple builtin methods.
* Builtin syntax checker. Any invalid inputs for policy headers or rules are rejected to be applied.
* Enhanced search. Any policy blocks in `Acl` object can be searched with its priority, operation or regex pattern of the line.
* I/O support. You can read policy from a file or plain text, and write policy into file, plain text or JSON.

## Usage Example

* [acquery](https://github.com/g1eng/acquery)

## Limitation

> [!WARNING]
> Not for production.

This is an artifact of a PoC project in three years ago. 
Its syntax tree is built with regex and has poor performance to analyze bigger policy files.

## Author 

[youmeim](https://github.com/g1eng)
