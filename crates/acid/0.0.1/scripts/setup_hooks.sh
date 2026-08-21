#!/bin/sh

echo "setting up git hooks"

ln -sf ../../scripts/pre_commit.sh .git/hooks/pre-commit
echo "pre-commit hook set up"

ln -sf ../../scripts/commit_msg.sh .git/hooks/commit-msg
echo "commit-msg hook set up"