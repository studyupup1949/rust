#!/bin/bash

wget https://github.com/jqlang/jq/releases/download/jq-1.8.1/jq-linux-amd64

chmod +x jq-linux-amd64

mv jq-linux-amd64 jq

export CRATE_NAME="$(cargo metadata --no-deps --format-version 1 | jq .packages[0].name | tr -d '\"')"
export PUBLISHED_VERSION="$(cargo search $CRATE_NAME --limit 1 | sed 's/.*"\(.*\)".*/\1/')"
export BRANCH_VERSION="$(cargo metadata --no-deps --format-version 1 | ./jq .packages[0].version | tr -d '"')"

echo "Currently published crates.io version: $PUBLISHED_VERSION"
echo "Branch version: $BRANCH_VERSION"

if [[ "$PUBLISHED_VERSION" != "$BRANCH_VERSION" ]]; then
    echo "Version Bumped."
    exit 0
else
    echo "Version in crates.io ($PUBLISHED_VERSION) exactly matches ($BRANCH_VERSION), cannot merge."
    exit 1
fi
